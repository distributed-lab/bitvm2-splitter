
// #[test]
// pub fn test_verify() {
//     assert!(Message::verify_random());
// }

// #[test]
// pub fn test_encode_restore() {
//     // Create a function

//     let msg = Message::from_u32(0x2FEEDDCC);
//     let recovery_script = Message::recovery_script();

//     let script = script! {
//         for part in msg.0.iter().take(N0).rev() {
//             { *part }
//         }

//         { recovery_script }
//         0x2FEEDDCC
//         OP_EQUAL
//     };

//     let result = execute_script(script);

//     println!("{}", result);

//     assert!(result.success);
// }
