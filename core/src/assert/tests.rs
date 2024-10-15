use crate::treepp::*;
use bitcoin_splitter::split::intermediate_state::IntermediateState;

use super::signing::SignedIntermediateState;

#[test]
pub fn test_stack_sign_and_verify() {
    // Define the test intermediate state
    let state = IntermediateState::from_input_script(
        &script!{}, 
        &script!{
        OP_2 OP_3 OP_4
    });
    
    // Now, we sign the state
    let signed_state = SignedIntermediateState::sign(state);

    // Check that witness + verification scripts are correct
    let verify_script = script! {
        { signed_state.witness_script() }
        { signed_state.verification_script() }
        OP_4 OP_EQUALVERIFY
        OP_3 OP_EQUALVERIFY
        OP_2 OP_EQUALVERIFY
        OP_TRUE
    };

    let result = execute_script(verify_script);
    assert!(result.success, "Verification failed");
}

#[test]
pub fn test_stack_sign_and_verify_with_altstack() {
    // Define the test intermediate state
    let state = IntermediateState::from_input_script(
        &script!{}, 
        &script!{
        { 2345 } OP_3 { 1636 }
        OP_5 OP_TOALTSTACK
        OP_6 OP_TOALTSTACK
        OP_7 OP_TOALTSTACK
    });
    
    // Now, we sign the state
    let signed_state = SignedIntermediateState::sign(state);

    // Check that witness + verification scripts are correct
    let verify_script = script! {
        { signed_state.witness_script() }
        { signed_state.verification_script() }
        { 1636 } OP_EQUALVERIFY
        OP_3 OP_EQUALVERIFY
        { 2345 } OP_EQUALVERIFY
        OP_FROMALTSTACK OP_5 OP_EQUALVERIFY
        OP_FROMALTSTACK OP_6 OP_EQUALVERIFY
        OP_FROMALTSTACK OP_7 OP_EQUALVERIFY
        OP_TRUE
    };

    for state in signed_state.stack {
        println!("Stack element: {:?}", state.public_key);
    }

    let result = execute_script(verify_script);
    assert!(result.success, "Verification failed");
}
