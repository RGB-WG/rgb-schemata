fn main() {
    // 2. Prepare transition
    let mut main_inputs = Vec::<XOutputSeal>::new();
    let mut sum_inputs = Amount::ZERO;
    let mut sum_alt = Amount::ZERO;
    let mut data_inputs = vec![];
    let mut data_main = true;
    let lookup_state =
        if let InvoiceState::Data(NonFungible::RGB21(allocation)) = &invoice.owned_state {
            Some(DataState::from(*allocation))
        } else {
            None
        };

    for (output, list) in
        self.contract_assignments_for(contract_id, prev_outputs.iter().copied())?
    {
        if output.method() == method {
            main_inputs.push(output)
        } else {
            alt_inputs.push(output)
        };
        for (opout, mut state) in list {
            if output.method() == method {
                main_builder = main_builder.add_input(opout, state.clone())?;
            } else {
                alt_builder = alt_builder.add_input(opout, state.clone())?;
            }
            if opout.ty != assignment_id {
                let seal = output_for_assignment(contract_id, opout.ty)?;
                state.update_blinding(pedersen_blinder(contract_id, assignment_id));
                if output.method() == method {
                    main_builder = main_builder.add_owned_state_raw(opout.ty, seal, state)?;
                } else {
                    alt_builder = alt_builder.add_owned_state_raw(opout.ty, seal, state)?;
                }
            } else if let PersistedState::Amount(value, _, _) = state {
                sum_inputs += value;
                if output.method() != method {
                    sum_alt += value;
                }
            } else if let PersistedState::Data(value, _) = state {
                if lookup_state.as_ref() == Some(&value) && output.method() != method {
                    data_main = false;
                }
                data_inputs.push(value);
            }
        }
    }
    // Add payments to beneficiary and change
    match invoice.owned_state.clone() {
        InvoiceState::Amount(amt) => {
            // Pay beneficiary
            if sum_inputs < amt {
                return Err(ComposeError::InsufficientState.into());
            }

            let sum_main = sum_inputs - sum_alt;
            let (paid_main, paid_alt) =
                if sum_main < amt { (sum_main, amt - sum_main) } else { (amt, Amount::ZERO) };
            let blinding_beneficiary = pedersen_blinder(contract_id, assignment_id);

            if paid_main > Amount::ZERO {
                main_builder = main_builder.add_fungible_state_raw(
                    assignment_id,
                    beneficiary,
                    paid_main,
                    blinding_beneficiary,
                )?;
            }
            if paid_alt > Amount::ZERO {
                alt_builder = alt_builder.add_fungible_state_raw(
                    assignment_id,
                    beneficiary,
                    paid_alt,
                    blinding_beneficiary,
                )?;
            }

            let blinding_change = pedersen_blinder(contract_id, assignment_id);
            let change_seal = output_for_assignment(contract_id, assignment_id)?;

            // Pay change
            if sum_main > paid_main {
                main_builder = main_builder.add_fungible_state_raw(
                    assignment_id,
                    change_seal,
                    sum_main - paid_main,
                    blinding_change,
                )?;
            }
            if sum_alt > paid_alt {
                alt_builder = alt_builder.add_fungible_state_raw(
                    assignment_id,
                    change_seal,
                    sum_alt - paid_alt,
                    blinding_change,
                )?;
            }
        }
        InvoiceState::Data(data) => match data {
            NonFungible::RGB21(allocation) => {
                let lookup_state = DataState::from(allocation);
                if !data_inputs.into_iter().any(|x| x == lookup_state) {
                    return Err(ComposeError::InsufficientState.into());
                }

                let seal = seal_blinder(contract_id, assignment_id);
                if data_main {
                    main_builder = main_builder.add_data_raw(
                        assignment_id,
                        beneficiary,
                        allocation,
                        seal,
                    )?;
                } else {
                    alt_builder = alt_builder.add_data_raw(
                        assignment_id,
                        beneficiary,
                        allocation,
                        seal,
                    )?;
                }
            }
        },
        _ => {
            todo!(
                "only PersistedState::Amount and PersistedState::Allocation are currently \
                     supported"
            )
        }
    }
}