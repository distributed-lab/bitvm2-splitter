use crate::treepp::*;

/// Checks whether two elements, consisting of multiple limbs, are equal.
///
/// - **Input:** `{ a[0], a[1], ..., a[l-1], b[0], b[1], ..., b[l-1] }`
/// - **Returns:** `{ a[0] == b[0] && a[1] == b[1] && ... && a[l-1] == b[l-1] }`
#[allow(non_snake_case)]
pub fn OP_LONGEQUALVERIFY(length: usize) -> Script {
    script! {
        for j in (0..length).into_iter().rev() {
            { j+1 } OP_ROLL OP_EQUALVERIFY
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(non_snake_case)]
    fn test_OP_LONGEQUALVERIFY() {
        let script = script! {
            1 2 3 4 5 1 2 3 4 5
            { OP_LONGEQUALVERIFY(5) }
            OP_TRUE
        };

        let exec_result = execute_script(script);
        assert!(exec_result.success, "OP_LONGEQUALVERIFY(5) failed");
    }
}
