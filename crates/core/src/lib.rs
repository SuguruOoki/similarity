pub mod apted;
pub mod function_extractor;
pub mod parser;
pub mod tree;
pub mod tsed;
pub mod type_comparator;
pub mod type_extractor;
pub mod type_normalizer;

pub use apted::{compute_edit_distance, APTEDOptions};
pub use function_extractor::{
    compare_functions, extract_functions, find_similar_functions_across_files,
    find_similar_functions_in_file, FunctionDefinition, FunctionType, SimilarityResult,
};
pub use parser::{ast_to_tree_node, parse_and_convert_to_tree};
pub use tree::TreeNode;
pub use tsed::{calculate_tsed, calculate_tsed_from_code, TSEDOptions};

// Type-related exports
pub use type_comparator::{
    compare_type_literal_with_type, compare_types, find_duplicate_types,
    find_similar_type_literals, find_similar_type_literals_pairs, find_similar_types,
    group_similar_types, MatchedProperty, SimilarTypePair, TypeComparisonOptions,
    TypeComparisonResult, TypeDifferences, TypeLiteralComparisonPair, TypeMismatch,
};
pub use type_extractor::{
    extract_type_literals_from_code, extract_type_literals_from_files, extract_types_from_code,
    extract_types_from_files, PropertyDefinition, TypeDefinition, TypeKind, TypeLiteralContext,
    TypeLiteralDefinition,
};
pub use type_normalizer::{
    calculate_property_similarity, calculate_type_similarity, find_property_matches,
    normalize_type, NormalizationOptions, NormalizedType, PropertyMatch,
};
