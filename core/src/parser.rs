//! Pest-based parser for LinkML schemas
//!
//! This module provides a parser that converts LinkML YAML syntax into
//! Abstract Syntax Tree (AST) structures. The parser uses the Pest parsing
//! library with a grammar defined in `../../grammar/linkml.pest`.

// Allow missing docs for Pest-generated code
#![allow(missing_docs)]

use indexmap::IndexMap;
use pest::Parser;
use pest_derive::Parser;

use crate::ast::*;
use crate::error::{LinkMLError, Result};

/// Pest parser for LinkML YAML syntax
#[allow(missing_docs)]
#[derive(Parser)]
#[grammar = "../../grammar/linkml.pest"]
pub struct LinkMLParser;

/// Type alias for Pest parsing pairs
type Pair<'i> = pest::iterators::Pair<'i, Rule>;

impl LinkMLParser {
    /// Parse a complete LinkML schema from YAML string
    ///
    /// # Arguments
    ///
    /// * `input` - The YAML content as a string slice
    ///
    /// # Returns
    ///
    /// Returns a `Result<SchemaAst>` containing the parsed schema AST.
    ///
    /// # Errors
    ///
    /// Returns `LinkMLError::ParseError` if the input is not valid LinkML YAML.
    pub fn parse_schema(input: &str) -> Result<SchemaAst> {
        let pairs = Self::parse(Rule::schema, input)?;

        let mut schema = SchemaAst::new();
        schema.document_type = Some(DocumentType::Schema);

        for pair in pairs {
            match pair.as_rule() {
                Rule::schema => {
                    for inner_pair in pair.into_inner() {
                        Self::process_schema_field(&mut schema, inner_pair)?;
                    }
                }
                Rule::EOI => break,
                _ => {
                    return Err(LinkMLError::parse(format!(
                        "Unexpected rule: {:?}",
                        pair.as_rule()
                    )));
                }
            }
        }

        Ok(schema)
    }

    /// Helper function to create a `Spanned<T>` from a Pest pair
    fn create_spanned<T>(pair: &Pair<'_>, value: T) -> Spanned<T> {
        let span_info = pair.as_span();
        let (line, column) = span_info.start_pos().line_col();
        let span = Span::new(
            span_info.start(),
            span_info.end(),
            line,
            column,
        );
        Spanned::new(value, span)
    }

    /// Process a top-level schema field
    fn process_schema_field(schema: &mut SchemaAst, pair: Pair<'_>) -> Result<()> {
        match pair.as_rule() {
            Rule::schema_id => {
                for inner in pair.into_inner() {
                    if inner.as_rule() == Rule::uri {
                        schema.id = Some(Self::create_spanned(&inner, inner.as_str().to_string()));
                    }
                }
            }
            Rule::schema_name => {
                for inner in pair.into_inner() {
                    if inner.as_rule() == Rule::identifier {
                        schema.name = Some(Self::create_spanned(&inner, inner.as_str().to_string()));
                    }
                }
            }
            Rule::schema_title => {
                for inner in pair.into_inner() {
                    if matches!(inner.as_rule(), Rule::string_value | Rule::identifier) {
                        schema.title = Some(Self::create_spanned(&inner, Self::parse_string_value(inner.as_str())));
                    }
                }
            }
            Rule::schema_description => {
                for inner in pair.into_inner() {
                    if matches!(inner.as_rule(), Rule::string_value | Rule::block_string) {
                        let desc = Self::parse_description(&inner)?;
                        schema.description = Some(Self::create_spanned(&inner, desc));
                    }
                }
            }
            Rule::schema_version => {
                for inner in pair.into_inner() {
                    if inner.as_rule() == Rule::string_value {
                        schema.version = Some(Self::create_spanned(&inner, Self::parse_string_value(inner.as_str())));
                    }
                }
            }
            Rule::schema_license => {
                for inner in pair.into_inner() {
                    if inner.as_rule() == Rule::string_value {
                        schema.license = Some(Self::create_spanned(&inner, Self::parse_string_value(inner.as_str())));
                    }
                }
            }
            Rule::schema_created_on => {
                for inner in pair.into_inner() {
                    if inner.as_rule() == Rule::timestamp {
                        schema.created_on = Some(Self::create_spanned(&inner, inner.as_str().to_string()));
                    }
                }
            }
            Rule::schema_last_updated_on => {
                for inner in pair.into_inner() {
                    if inner.as_rule() == Rule::timestamp {
                        schema.last_updated_on = Some(Self::create_spanned(&inner, inner.as_str().to_string()));
                    }
                }
            }
            Rule::schema_default_prefix => {
                for inner in pair.into_inner() {
                    if inner.as_rule() == Rule::identifier {
                        schema.default_prefix = Some(Self::create_spanned(&inner, inner.as_str().to_string()));
                    }
                }
            }
            Rule::schema_default_range => {
                for inner in pair.into_inner() {
                    if inner.as_rule() == Rule::identifier {
                        schema.default_range = Some(Self::create_spanned(&inner, inner.as_str().to_string()));
                    }
                }
            }
            Rule::schema_metamodel_version => {
                for inner in pair.into_inner() {
                    if inner.as_rule() == Rule::string_value {
                        schema.metamodel_version = Some(Self::create_spanned(&inner, Self::parse_string_value(inner.as_str())));
                    }
                }
            }
            Rule::schema_source_file => {
                for inner in pair.into_inner() {
                    if inner.as_rule() == Rule::string_value {
                        schema.source_file = Some(Self::create_spanned(&inner, Self::parse_string_value(inner.as_str())));
                    }
                }
            }
            Rule::schema_generation_date => {
                for inner in pair.into_inner() {
                    if inner.as_rule() == Rule::timestamp {
                        schema.generation_date = Some(Self::create_spanned(&inner, inner.as_str().to_string()));
                    }
                }
            }
            Rule::schema_status => {
                for inner in pair.into_inner() {
                    if inner.as_rule() == Rule::string_value {
                        schema.status = Some(Self::create_spanned(&inner, Self::parse_string_value(inner.as_str())));
                    }
                }
            }
            Rule::schema_prefixes => {
                schema.prefixes = Self::parse_prefixes(pair)?;
            }
            Rule::schema_imports => {
                schema.imports = Self::parse_imports(pair)?;
            }
            Rule::schema_settings => {
                schema.settings = Self::parse_settings(pair)?;
            }
            Rule::schema_classes => {
                schema.classes = Self::parse_classes(pair)?;
            }
            Rule::schema_slots => {
                schema.slots = Self::parse_slots(pair)?;
            }
            Rule::schema_types => {
                schema.types = Self::parse_types(pair)?;
            }
            Rule::schema_enums => {
                schema.enums = Self::parse_enums(pair)?;
            }
            Rule::schema_subsets => {
                schema.subsets = Self::parse_subsets(pair)?;
            }
            Rule::schema_contributors => {
                schema.contributors = Self::parse_contributors(pair)?;
            }
            Rule::schema_categories => {
                schema.categories = Self::parse_string_list(pair)?;
            }
            Rule::schema_keywords => {
                schema.keywords = Self::parse_string_list(pair)?;
            }
            Rule::schema_see_also => {
                schema.see_also = Self::parse_string_list(pair)?;
            }
            Rule::schema_annotations => {
                // Process annotation entries directly from schema_annotations
                let annotations = Self::parse_annotations(pair.clone())?;
                schema.annotations = Some(Self::create_spanned(&pair, annotations));
            }
            _ => {
                // Ignore unknown fields or continue
            }
        }
        Ok(())
    }

    /// Parse string value, removing quotes if present
    fn parse_string_value(s: &str) -> String {
        s.trim()
            .trim_matches('"')
            .trim_matches('\'')
            .to_string()
    }

    /// Parse description (inline or block string)
    fn parse_description(pair: &Pair<'_>) -> Result<Description> {
        match pair.as_rule() {
            Rule::string_value => Ok(Description::Inline(Self::parse_string_value(pair.as_str()))),
            Rule::block_string => Ok(Description::Block(pair.as_str().trim().to_string())),
            _ => Err(LinkMLError::parse("Invalid description format")),
        }
    }

    /// Parse prefixes section
    fn parse_prefixes(pair: Pair<'_>) -> Result<IndexMap<String, Spanned<String>>> {
        let mut prefixes = IndexMap::new();
        for inner in pair.into_inner() {
            if inner.as_rule() == Rule::prefix_entry {
                let mut parts = inner.into_inner();
                if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                    if key.as_rule() == Rule::identifier && value.as_rule() == Rule::uri {
                        let key_str = key.as_str().to_string();
                        let value_spanned = Self::create_spanned(&value, value.as_str().to_string());
                        prefixes.insert(key_str, value_spanned);
                    }
                }
            }
        }
        Ok(prefixes)
    }

    /// Parse imports section
    fn parse_imports(pair: Pair<'_>) -> Result<Vec<Spanned<String>>> {
        let mut imports = Vec::new();
        for inner in pair.into_inner() {
            if inner.as_rule() == Rule::import_entry {
                for item in inner.into_inner() {
                    if matches!(item.as_rule(), Rule::uri | Rule::identifier) {
                        imports.push(Self::create_spanned(&item, item.as_str().to_string()));
                    }
                }
            }
        }
        Ok(imports)
    }

    /// Parse settings section
    fn parse_settings(pair: Pair<'_>) -> Result<IndexMap<String, Spanned<String>>> {
        let mut settings = IndexMap::new();
        for inner in pair.into_inner() {
            if inner.as_rule() == Rule::setting_entry {
                let mut parts = inner.into_inner();
                if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                    if key.as_rule() == Rule::identifier {
                        let key_str = key.as_str().to_string();
                        let value_str = match value.as_rule() {
                            Rule::pattern_value | Rule::string_value => {
                                Self::parse_string_value(value.as_str())
                            }
                            _ => value.as_str().to_string(),
                        };
                        settings.insert(key_str, Self::create_spanned(&value, value_str));
                    }
                }
            }
        }
        Ok(settings)
    }

    /// Parse classes section
    fn parse_classes(pair: Pair<'_>) -> Result<IndexMap<String, Spanned<ClassAst>>> {
        let mut classes = IndexMap::new();
        for inner in pair.into_inner() {
            if inner.as_rule() == Rule::class_definition {
                let mut parts = inner.into_inner();
                if let Some(name_pair) = parts.next() {
                    if name_pair.as_rule() == Rule::identifier {
                        let name = name_pair.as_str().to_string();
                        let mut class_ast = ClassAst {
                            name: name.clone(),
                            ..Default::default()
                        };

                        // Process class fields
                        for field in parts {
                            Self::process_class_field(&mut class_ast, field)?;
                        }

                        classes.insert(name, Self::create_spanned(&name_pair, class_ast));
                    }
                }
            }
        }
        Ok(classes)
    }

    /// Process a class field
    fn process_class_field(class: &mut ClassAst, pair: Pair<'_>) -> Result<()> {
        match pair.as_rule() {
            Rule::class_description => {
                for inner in pair.into_inner() {
                    if matches!(inner.as_rule(), Rule::string_value | Rule::block_string) {
                        let desc = Self::parse_description(&inner)?;
                        class.description = Some(Self::create_spanned(&inner, desc));
                    }
                }
            }
            Rule::class_is_a => {
                for inner in pair.into_inner() {
                    if inner.as_rule() == Rule::identifier {
                        class.is_a = Some(Self::create_spanned(&inner, inner.as_str().to_string()));
                    }
                }
            }
            Rule::class_abstract => {
                for inner in pair.into_inner() {
                    if inner.as_rule() == Rule::boolean {
                        let value = inner.as_str() == "true";
                        class.abstract_ = Some(Self::create_spanned(&inner, value));
                    }
                }
            }
            Rule::class_mixin => {
                for inner in pair.into_inner() {
                    if inner.as_rule() == Rule::boolean {
                        let value = inner.as_str() == "true";
                        class.mixin = Some(Self::create_spanned(&inner, value));
                    }
                }
            }
            Rule::class_tree_root => {
                for inner in pair.into_inner() {
                    if inner.as_rule() == Rule::boolean {
                        let value = inner.as_str() == "true";
                        class.tree_root = Some(Self::create_spanned(&inner, value));
                    }
                }
            }
            Rule::class_class_uri => {
                for inner in pair.into_inner() {
                    if inner.as_rule() == Rule::uri {
                        class.class_uri = Some(Self::create_spanned(&inner, inner.as_str().to_string()));
                    }
                }
            }
            Rule::class_mixins => {
                class.mixins = Self::parse_string_list(pair)?;
            }
            Rule::class_slots => {
                class.slots = Self::parse_string_list(pair)?;
            }
            Rule::class_aliases => {
                class.aliases = Self::parse_string_list(pair)?;
            }
            Rule::class_see_also => {
                class.see_also = Self::parse_string_list(pair)?;
            }
            Rule::class_id_prefixes => {
                class.id_prefixes = Self::parse_string_list(pair)?;
            }
            Rule::class_broad_mappings => {
                class.broad_mappings = Self::parse_string_list(pair)?;
            }
            Rule::class_exact_mappings => {
                class.exact_mappings = Self::parse_string_list(pair)?;
            }
            Rule::class_narrow_mappings => {
                class.narrow_mappings = Self::parse_string_list(pair)?;
            }
            Rule::class_related_mappings => {
                class.related_mappings = Self::parse_string_list(pair)?;
            }
            Rule::class_close_mappings => {
                class.close_mappings = Self::parse_string_list(pair)?;
            }
            Rule::class_subclass_of => {
                class.subclass_of = Self::parse_string_list(pair)?;
            }
            _ => {
                // Ignore unknown fields
            }
        }
        Ok(())
    }

    /// Parse slots section
    fn parse_slots(pair: Pair<'_>) -> Result<IndexMap<String, Spanned<SlotAst>>> {
        let mut slots = IndexMap::new();
        for inner in pair.into_inner() {
            if inner.as_rule() == Rule::slot_definition {
                let mut parts = inner.into_inner();
                if let Some(name_pair) = parts.next() {
                    if name_pair.as_rule() == Rule::identifier {
                        let name = name_pair.as_str().to_string();
                        let mut slot_ast = SlotAst {
                            name: name.clone(),
                            ..Default::default()
                        };

                        // Process slot fields
                        for field in parts {
                            Self::process_slot_field(&mut slot_ast, field)?;
                        }

                        slots.insert(name, Self::create_spanned(&name_pair, slot_ast));
                    }
                }
            }
        }
        Ok(slots)
    }

    /// Process a slot field
    fn process_slot_field(slot: &mut SlotAst, pair: Pair<'_>) -> Result<()> {
        match pair.as_rule() {
            Rule::slot_description => {
                for inner in pair.into_inner() {
                    if matches!(inner.as_rule(), Rule::string_value | Rule::block_string) {
                        let desc = Self::parse_description(&inner)?;
                        slot.description = Some(Self::create_spanned(&inner, desc));
                    }
                }
            }
            Rule::slot_range => {
                for inner in pair.into_inner() {
                    if inner.as_rule() == Rule::identifier {
                        slot.range = Some(Self::create_spanned(&inner, inner.as_str().to_string()));
                    }
                }
            }
            Rule::slot_required => {
                for inner in pair.into_inner() {
                    if inner.as_rule() == Rule::boolean {
                        let value = inner.as_str() == "true";
                        slot.required = Some(Self::create_spanned(&inner, value));
                    }
                }
            }
            Rule::slot_multivalued => {
                for inner in pair.into_inner() {
                    if inner.as_rule() == Rule::boolean {
                        let value = inner.as_str() == "true";
                        slot.multivalued = Some(Self::create_spanned(&inner, value));
                    }
                }
            }
            Rule::slot_identifier => {
                for inner in pair.into_inner() {
                    if inner.as_rule() == Rule::boolean {
                        let value = inner.as_str() == "true";
                        slot.identifier = Some(Self::create_spanned(&inner, value));
                    }
                }
            }
            Rule::slot_pattern => {
                for inner in pair.into_inner() {
                    if matches!(inner.as_rule(), Rule::string_value | Rule::pattern_value) {
                        slot.pattern = Some(Self::create_spanned(&inner, Self::parse_string_value(inner.as_str())));
                    }
                }
            }
            Rule::slot_is_a => {
                for inner in pair.into_inner() {
                    if inner.as_rule() == Rule::identifier {
                        slot.is_a = Some(Self::create_spanned(&inner, inner.as_str().to_string()));
                    }
                }
            }
            Rule::slot_domain => {
                for inner in pair.into_inner() {
                    if inner.as_rule() == Rule::identifier {
                        slot.domain = Some(Self::create_spanned(&inner, inner.as_str().to_string()));
                    }
                }
            }
            Rule::slot_inverse => {
                for inner in pair.into_inner() {
                    if inner.as_rule() == Rule::identifier {
                        slot.inverse = Some(Self::create_spanned(&inner, inner.as_str().to_string()));
                    }
                }
            }
            Rule::slot_symmetric => {
                for inner in pair.into_inner() {
                    if inner.as_rule() == Rule::boolean {
                        let value = inner.as_str() == "true";
                        slot.symmetric = Some(Self::create_spanned(&inner, value));
                    }
                }
            }
            Rule::slot_mixins => {
                slot.mixins = Self::parse_string_list(pair)?;
            }
            Rule::slot_aliases => {
                slot.aliases = Self::parse_string_list(pair)?;
            }
            Rule::slot_see_also => {
                slot.see_also = Self::parse_string_list(pair)?;
            }
            _ => {
                // Ignore unknown fields
            }
        }
        Ok(())
    }

    /// Parse types section
    fn parse_types(pair: Pair<'_>) -> Result<IndexMap<String, Spanned<TypeAst>>> {
        let mut types = IndexMap::new();
        for inner in pair.into_inner() {
            if inner.as_rule() == Rule::type_definition {
                let mut parts = inner.into_inner();
                if let Some(name_pair) = parts.next() {
                    if name_pair.as_rule() == Rule::identifier {
                        let name = name_pair.as_str().to_string();
                        let mut type_ast = TypeAst {
                            name: name.clone(),
                            ..Default::default()
                        };

                        // Process type fields
                        for field in parts {
                            Self::process_type_field(&mut type_ast, field)?;
                        }

                        types.insert(name, Self::create_spanned(&name_pair, type_ast));
                    }
                }
            }
        }
        Ok(types)
    }

    /// Process a type field
    fn process_type_field(type_ast: &mut TypeAst, pair: Pair<'_>) -> Result<()> {
        match pair.as_rule() {
            Rule::type_description => {
                for inner in pair.into_inner() {
                    if matches!(inner.as_rule(), Rule::string_value | Rule::block_string) {
                        let desc = Self::parse_description(&inner)?;
                        type_ast.description = Some(Self::create_spanned(&inner, desc));
                    }
                }
            }
            Rule::type_typeof => {
                for inner in pair.into_inner() {
                    if inner.as_rule() == Rule::identifier {
                        type_ast.typeof_ = Some(Self::create_spanned(&inner, inner.as_str().to_string()));
                    }
                }
            }
            Rule::type_base => {
                for inner in pair.into_inner() {
                    if inner.as_rule() == Rule::identifier {
                        type_ast.base = Some(Self::create_spanned(&inner, inner.as_str().to_string()));
                    }
                }
            }
            Rule::type_uri => {
                for inner in pair.into_inner() {
                    if inner.as_rule() == Rule::uri {
                        type_ast.uri = Some(Self::create_spanned(&inner, inner.as_str().to_string()));
                    }
                }
            }
            Rule::type_pattern => {
                for inner in pair.into_inner() {
                    if matches!(inner.as_rule(), Rule::string_value | Rule::pattern_value) {
                        type_ast.pattern = Some(Self::create_spanned(&inner, Self::parse_string_value(inner.as_str())));
                    }
                }
            }
            _ => {
                // Ignore unknown fields
            }
        }
        Ok(())
    }

    /// Parse enums section
    fn parse_enums(pair: Pair<'_>) -> Result<IndexMap<String, Spanned<EnumAst>>> {
        let mut enums = IndexMap::new();
        for inner in pair.into_inner() {
            if inner.as_rule() == Rule::enum_definition {
                let mut parts = inner.into_inner();
                if let Some(name_pair) = parts.next() {
                    if name_pair.as_rule() == Rule::identifier {
                        let name = name_pair.as_str().to_string();
                        let mut enum_ast = EnumAst {
                            name: name.clone(),
                            ..Default::default()
                        };

                        // Process enum fields
                        for field in parts {
                            Self::process_enum_field(&mut enum_ast, field)?;
                        }

                        enums.insert(name, Self::create_spanned(&name_pair, enum_ast));
                    }
                }
            }
        }
        Ok(enums)
    }

    /// Process an enum field
    fn process_enum_field(enum_ast: &mut EnumAst, pair: Pair<'_>) -> Result<()> {
        match pair.as_rule() {
            Rule::enum_description => {
                for inner in pair.into_inner() {
                    if matches!(inner.as_rule(), Rule::string_value | Rule::block_string) {
                        let desc = Self::parse_description(&inner)?;
                        enum_ast.description = Some(Self::create_spanned(&inner, desc));
                    }
                }
            }
            Rule::enum_permissible_values => {
                enum_ast.permissible_values = Self::parse_permissible_values(pair)?;
            }
            _ => {
                // Ignore unknown fields
            }
        }
        Ok(())
    }

    /// Parse permissible values
    fn parse_permissible_values(pair: Pair<'_>) -> Result<IndexMap<String, Spanned<PermissibleValueAst>>> {
        let mut values = IndexMap::new();
        for inner in pair.into_inner() {
            if inner.as_rule() == Rule::permissible_value_entry {
                let mut parts = inner.into_inner();
                if let Some(name_pair) = parts.next() {
                    if name_pair.as_rule() == Rule::identifier {
                        let name = name_pair.as_str().to_string();
                        let pv = PermissibleValueAst {
                            name: name.clone(),
                            ..Default::default()
                        };
                        values.insert(name, Self::create_spanned(&name_pair, pv));
                    }
                }
            }
        }
        Ok(values)
    }

    /// Parse subsets section
    fn parse_subsets(pair: Pair<'_>) -> Result<IndexMap<String, Spanned<SubsetAst>>> {
        let mut subsets = IndexMap::new();
        for inner in pair.into_inner() {
            if inner.as_rule() == Rule::subset_definition {
                let mut parts = inner.into_inner();
                if let Some(name_pair) = parts.next() {
                    if name_pair.as_rule() == Rule::identifier {
                        let name = name_pair.as_str().to_string();
                        let subset = SubsetAst {
                            name: name.clone(),
                            ..Default::default()
                        };
                        subsets.insert(name, Self::create_spanned(&name_pair, subset));
                    }
                }
            }
        }
        Ok(subsets)
    }

    /// Parse contributors section
    fn parse_contributors(pair: Pair<'_>) -> Result<Vec<Spanned<ContributorAst>>> {
        let mut contributors = Vec::new();
        for inner in pair.into_inner() {
            if inner.as_rule() == Rule::contributor_entry {
                let name = inner.as_str().to_string();
                let contributor = ContributorAst {
                    name: name.clone(),
                    email: None,
                    orcid: None,
                };
                contributors.push(Self::create_spanned(&inner, contributor));
            }
        }
        Ok(contributors)
    }

    /// Parse a list of strings
    fn parse_string_list(pair: Pair<'_>) -> Result<Vec<Spanned<String>>> {
        let mut list = Vec::new();
        for inner in pair.into_inner() {
            if matches!(inner.as_rule(), Rule::identifier | Rule::string_value | Rule::uri) {
                list.push(Self::create_spanned(&inner, inner.as_str().to_string()));
            }
        }
        Ok(list)
    }

    /// Parse annotations
    fn parse_annotations(pair: Pair<'_>) -> Result<AnnotationsAst> {
        let mut annotations = AnnotationsAst::default();
        for inner in pair.into_inner() {
            if inner.as_rule() == Rule::annotation_entry {
                let mut parts = inner.into_inner();
                if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                    let key_str = key.as_str().to_string();
                    let value_ast = Self::parse_annotation_value(&value)?;
                    annotations.entries.insert(key_str, Self::create_spanned(&value, value_ast));
                }
            }
        }
        Ok(annotations)
    }

    /// Parse annotation value
    fn parse_annotation_value(pair: &Pair<'_>) -> Result<AnnotationValueAst> {
        match pair.as_rule() {
            Rule::boolean => {
                let value = pair.as_str() == "true";
                Ok(AnnotationValueAst::Bool(value))
            }
            Rule::number => {
                let value = pair.as_str().parse::<f64>()
                    .map_err(|_| LinkMLError::parse("Invalid number in annotation"))?;
                Ok(AnnotationValueAst::Number(value))
            }
            Rule::string_value => {
                Ok(AnnotationValueAst::String(Self::parse_string_value(pair.as_str())))
            }
            Rule::block_string => {
                Ok(AnnotationValueAst::Block(pair.as_str().trim().to_string()))
            }
            _ => Err(LinkMLError::parse("Invalid annotation value type")),
        }
    }
}
