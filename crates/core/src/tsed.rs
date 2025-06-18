use crate::apted::{compute_edit_distance, APTEDOptions};
use crate::tree::TreeNode;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct TSEDOptions {
    pub apted_options: APTEDOptions,
    pub min_lines: u32,  // Minimum number of lines for a function to be considered
    pub size_penalty: bool,  // Apply penalty for short functions
}

impl Default for TSEDOptions {
    fn default() -> Self {
        TSEDOptions {
            apted_options: APTEDOptions {
                rename_cost: 0.3, // Default from the TypeScript implementation
                delete_cost: 1.0,
                insert_cost: 1.0,
            },
            min_lines: 5, // Increased default to better filter trivial matches
            size_penalty: true, // Enable size penalty by default
        }
    }
}

/// Calculate TSED (Tree Structure Edit Distance) similarity between two trees
/// Returns a value between 0.0 and 1.0, where 1.0 means identical
#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn calculate_tsed(tree1: &Rc<TreeNode>, tree2: &Rc<TreeNode>, options: &TSEDOptions) -> f64 {
    let distance = compute_edit_distance(tree1, tree2, &options.apted_options);

    let size1 = tree1.get_subtree_size() as f64;
    let size2 = tree2.get_subtree_size() as f64;
    
    // TSED normalization: Use the larger tree size
    // This ensures that when comparing trees of different sizes,
    // the similarity reflects how much of the larger tree matches
    let max_size = size1.max(size2);
    
    // Calculate base TSED similarity
    let tsed_similarity = if max_size > 0.0 {
        (1.0 - distance / max_size).max(0.0)
    } else {
        1.0
    };
    
    // Apply additional penalties for structural differences
    let mut similarity = tsed_similarity;
    
    // Size ratio penalty: penalize when trees have very different sizes
    let size_ratio = size1.min(size2) / size1.max(size2);
    
    if options.size_penalty {
        // For short functions, make differences more pronounced
        let min_size = size1.min(size2);
        
        if min_size < 20.0 {
            // Short function penalty: the shorter, the more sensitive to differences
            let short_function_factor = (min_size / 20.0).powf(0.5);
            similarity *= short_function_factor;
        }
        
        // Size difference penalty
        if size_ratio < 0.5 {
            // If one tree is less than half the size of the other,
            // they're likely fundamentally different
            similarity *= size_ratio.powf(0.5);
        }
    }
    
    similarity
}

/// Calculate TSED from TypeScript code strings
///
/// # Errors
///
/// Returns an error if parsing fails for either code string
pub fn calculate_tsed_from_code(
    code1: &str,
    code2: &str,
    filename1: &str,
    filename2: &str,
    options: &TSEDOptions,
) -> Result<f64, String> {
    use crate::parser::parse_and_convert_to_tree;

    let tree1 = parse_and_convert_to_tree(filename1, code1)?;
    let tree2 = parse_and_convert_to_tree(filename2, code2)?;

    Ok(calculate_tsed(&tree1, &tree2, options))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identical_code() {
        let code = "function add(a: number, b: number) { return a + b; }";
        let mut options = TSEDOptions::default();
        options.size_penalty = false; // Disable for small test functions

        let similarity =
            calculate_tsed_from_code(code, code, "test1.ts", "test2.ts", &options).unwrap();
        assert!((similarity - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_renamed_function() {
        let code1 = "function add(a: number, b: number) { return a + b; }";
        let code2 = "function sum(x: number, y: number) { return x + y; }";
        let mut options = TSEDOptions::default();
        options.size_penalty = false; // Disable for small test functions

        let similarity =
            calculate_tsed_from_code(code1, code2, "test1.ts", "test2.ts", &options).unwrap();
        // Should have high similarity due to low rename cost
        assert!(similarity > 0.8);
    }

    #[test]
    fn test_different_structure() {
        let code1 = "function test() { return 1; }";
        let code2 = "class Test { method() { return 1; } }";
        let options = TSEDOptions::default();

        let similarity =
            calculate_tsed_from_code(code1, code2, "test1.ts", "test2.ts", &options).unwrap();
        // Should have lower similarity due to structural differences
        assert!(similarity < 0.7);
    }
}
