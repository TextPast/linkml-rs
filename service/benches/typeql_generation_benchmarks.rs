//! Benchmarks for `TypeQL` generation performance.

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use indexmap::IndexMap;
use linkml_core::types::{
    ClassDefinition, ConditionalRequirement, Rule, RuleConditions, SchemaDefinition, SlotCondition,
    SlotDefinition,
};
use linkml_service::generator::{
    Generator, GeneratorOptions, IndentStyle, typeql_generator_enhanced::EnhancedTypeQLGenerator,
};
use std::fmt::Display;

/// Helper that panics with context when a benchmark setup step fails.
fn require_ok<T, E>(result: Result<T, E>, context: &str) -> T
where
    E: Display,
{
    match result {
        Ok(value) => value,
        Err(err) => panic!("{context}: {err}"),
    }
}

/// Create a simple schema with the specified number of classes
fn create_schema(num_classes: usize) -> SchemaDefinition {
    let mut schema = SchemaDefinition {
        name: format!("BenchmarkSchema{}", num_classes),
        id: format!("https://example.org/schemas/benchmark{}", num_classes),
        ..Default::default()
    };

    // Add common slots
    let name_slot = SlotDefinition {
        range: Some("string".to_string()),
        required: Some(true),
        ..Default::default()
    };
    schema.slots.insert("name".to_string(), name_slot);

    let id_slot = SlotDefinition {
        range: Some("string".to_string()),
        identifier: Some(true),
        ..Default::default()
    };
    schema.slots.insert("id".to_string(), id_slot);

    let created_at = SlotDefinition {
        range: Some("datetime".to_string()),
        ..Default::default()
    };
    schema.slots.insert("created_at".to_string(), created_at);

    // Create classes
    for i in 0..num_classes {
        let mut class = ClassDefinition {
            description: Some(format!("Test class {}", i)),
            ..Default::default()
        };

        // Add slots
        class.slots.extend(vec![
            "id".to_string(),
            "name".to_string(),
            "created_at".to_string(),
        ]);

        // Add some classes with inheritance
        if i > 0 && i % 3 == 0 {
            class.is_a = Some(format!("Class{}", i - 1));
        }

        // Add some relations
        if i % 5 == 0 && i > 0 {
            class.slots.push(format!("related_to_{}", i - 1));

            let rel_slot = SlotDefinition {
                range: Some(format!("Class{}", i - 1)),
                multivalued: Some(true),
                ..Default::default()
            };
            schema
                .slots
                .insert(format!("related_to_{}", i - 1), rel_slot);
        }

        schema.classes.insert(format!("Class{}", i), class);
    }

    schema
}

/// Create a schema with complex relationships
fn create_complex_schema(num_relations: usize) -> SchemaDefinition {
    let mut schema = SchemaDefinition {
        name: "ComplexRelationshipSchema".to_string(),
        ..Default::default()
    };

    // Create base entity classes
    for i in 0..5 {
        let class = ClassDefinition {
            slots: vec!["id".to_string()],
            ..Default::default()
        };
        schema.classes.insert(format!("Entity{i}"), class);
    }

    // Create relation classes
    for i in 0..num_relations {
        let mut rel_class = ClassDefinition {
            description: Some("Complex multi-way relation".to_string()),
            ..Default::default()
        };

        // Add 3-5 role players
        let num_roles = 3 + (i % 3);
        for j in 0..num_roles {
            rel_class.slots.push(format!("role_{}", j));

            let role_slot = SlotDefinition {
                range: Some(format!("Entity{}", j % 5)),
                ..Default::default()
            };
            schema
                .slots
                .insert(format!("relation_{}_role_{}", i, j), role_slot);
        }

        schema.classes.insert(format!("Relation{}", i), rel_class);
    }

    schema
}

/// Create a schema with many rules
fn create_rule_heavy_schema(num_rules: usize) -> SchemaDefinition {
    let mut schema = create_schema(10); // Base schema with 10 classes

    // Add rules to classes
    for (i, class) in schema.classes.values_mut().enumerate() {
        if i >= num_rules {
            break;
        }

        // Add validation rules
        let pre_slot_condition = SlotCondition {
            equals_string: Some("".to_string()),
            ..Default::default()
        };

        let mut pre_map = IndexMap::new();
        pre_map.insert("name".to_string(), pre_slot_condition);
        let preconditions = RuleConditions {
            slot_conditions: Some(pre_map),
            ..Default::default()
        };

        let post_slot_condition = SlotCondition {
            required: Some(true),
            ..Default::default()
        };

        let mut post_map = IndexMap::new();
        post_map.insert("created_at".to_string(), post_slot_condition);
        let postconditions = RuleConditions {
            slot_conditions: Some(post_map),
            ..Default::default()
        };

        let rule = Rule {
            preconditions: Some(preconditions),
            postconditions: Some(postconditions),
            ..Default::default()
        };

        class.rules.push(rule);

        // Add conditional requirements
        let condition = SlotCondition {
            equals_string: Some("special".to_string()),
            ..Default::default()
        };

        let requirement = ConditionalRequirement {
            condition: Some(condition),
            then_required: Some(vec!["created_at".to_string()]),
            ..Default::default()
        };

        let mut map = class.if_required.take().unwrap_or_default();
        map.insert("name".to_string(), requirement);
        class.if_required = Some(map);
    }

    schema
}

/// Benchmark generation for small schemas with simple relationships.
fn bench_simple_schemas(c: &mut Criterion) {
    let mut group = c.benchmark_group("typeql_simple_schemas");

    for size in [10, 50, 100, 500, 1000].iter() {
        let schema = create_schema(*size);
        let options = GeneratorOptions {
            indent: IndentStyle::Spaces(2),
            ..Default::default()
        };

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let generator = EnhancedTypeQLGenerator::with_options(options.clone());
                let output = require_ok(
                    generator.generate(black_box(&schema)),
                    "TypeQL generation should succeed",
                );
                black_box(output.len());
            });
        });
    }

    group.finish();
}

/// Benchmark generation for schemas containing complex relationships.
fn bench_complex_relations(c: &mut Criterion) {
    let mut group = c.benchmark_group("typeql_complex_relations");

    for num_relations in [10, 25, 50, 100].iter() {
        let schema = create_complex_schema(*num_relations);
        let options = GeneratorOptions::default();

        group.bench_with_input(
            BenchmarkId::from_parameter(num_relations),
            num_relations,
            |b, _| {
                b.iter(|| {
                    let generator = EnhancedTypeQLGenerator::with_options(options.clone());
                    let output = require_ok(
                        generator.generate(black_box(&schema)),
                        "TypeQL generation should succeed",
                    );
                    black_box(output.len());
                });
            },
        );
    }

    group.finish();
}

/// Benchmark generation of `TypeQL` rules from conditional requirements.
fn bench_rule_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("typeql_rule_generation");

    for num_rules in [10, 50, 100, 200].iter() {
        let schema = create_rule_heavy_schema(*num_rules);
        let options = GeneratorOptions::default();

        group.bench_with_input(BenchmarkId::from_parameter(num_rules), num_rules, |b, _| {
            b.iter(|| {
                let generator = EnhancedTypeQLGenerator::with_options(options.clone());
                let output = require_ok(
                    generator.generate(black_box(&schema)),
                    "TypeQL generation should succeed",
                );
                black_box(output.len());
            });
        });
    }

    group.finish();
}

/// Benchmark memory allocation patterns during generation.
fn bench_memory_allocation(c: &mut Criterion) {
    c.bench_function("typeql_string_allocation_1000_classes", |b| {
        let schema = create_schema(1000);
        let generator = EnhancedTypeQLGenerator::new();

        b.iter(|| {
            // This measures the full generation including string allocation
            let output = require_ok(
                generator.generate(black_box(&schema)),
                "TypeQL generation should succeed",
            );
            black_box(output.len());
        })
    });
}

criterion_group!(
    benches,
    bench_simple_schemas,
    bench_complex_relations,
    bench_rule_generation,
    bench_memory_allocation
);
criterion_main!(benches);
