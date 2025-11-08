# LinkML-Map Feature Comparison

**Last Updated**: 2025-11-08  
**Status**: 0% Parity - Feature Not Implemented

## Executive Summary

**linkml-map** is a separate Python package (`linkml-map`) that provides data transformation and mapping capabilities for LinkML schemas. RootReal's LinkML implementation does NOT include these features.

**Parity Score**: 0% (not implemented)  
**Impact**: Low-Medium (depends on use case)

## What is linkml-map?

linkml-map is a data transformation framework that allows:
- Mapping between different LinkML schemas
- Data transformation pipelines
- Schema-to-schema conversions
- ETL (Extract, Transform, Load) operations

**Repository**: https://github.com/linkml/linkml-map  
**Package**: `linkml-map` (separate from core `linkml`)

## Features RootReal Does NOT Have

### Data Transformation Pipeline
| Feature | linkml-map | RootReal | Impact |
|---------|-----------|----------|---------|
| Schema-to-schema mapping | ✅ | ❌ | High |
| Field transformations | ✅ | ❌ | High |
| Value transformations | ✅ | ❌ | High |
| Conditional mappings | ✅ | ❌ | Medium |
| Custom transformation functions | ✅ | ❌ | Medium |

### Mapping Definition Language
| Feature | linkml-map | RootReal | Impact |
|---------|-----------|----------|---------|
| Mapping schema YAML | ✅ | ❌ | High |
| Source/target schema refs | ✅ | ❌ | High |
| Transformation expressions | ✅ | ❌ | High |
| Mapping validation | ✅ | ❌ | Medium |

### Execution Engine
| Feature | linkml-map | RootReal | Impact |
|---------|-----------|----------|---------|
| Mapping executor | ✅ | ❌ | High |
| Batch transformations | ✅ | ❌ | Medium |
| Error handling | ✅ | ❌ | Medium |
| Transformation logging | ✅ | ❌ | Low |

## Why RootReal Doesn't Include This

**Architectural Separation**: linkml-map is a separate concern from schema validation and code generation. It's a specialized ETL/transformation layer.

**RootReal Focus**: RootReal focuses on:
1. Schema validation and enforcement
2. Code generation from schemas
3. Production-grade schema management
4. Performance and reliability

**Data transformation** is better handled by:
- Dedicated ETL tools
- Application-level mapping code
- Custom transformation services

## When You Need linkml-map Features

### Use Cases Requiring Transformation
1. **Schema migration**: Converting data from old schema to new schema
2. **System integration**: Mapping between different data models
3. **Data import/export**: Converting external data to internal schema
4. **Legacy system integration**: Bridging incompatible schemas

### Workarounds in RootReal
If you need transformation capabilities:

1. **Manual transformation code**:
   ```rust
   // Write custom Rust code to transform data
   fn transform_legacy_to_current(legacy: LegacyData) -> CurrentData {
       CurrentData {
           new_field: map_old_to_new(&legacy.old_field),
           // ... custom mapping logic
       }
   }
   ```

2. **Use external ETL tools**:
   - Apache NiFi
   - Airbyte
   - dbt (data build tool)

3. **GraphQL layer**:
   - Use GraphQL resolvers for on-the-fly transformations
   - Map between schemas in resolver logic

4. **Event streaming transformations**:
   - Use Fluvio SmartModules for stream transformations
   - Transform data in event pipelines

## Should RootReal Implement linkml-map Features?

### Arguments FOR Implementation
- **Completeness**: Would provide full LinkML ecosystem parity
- **Convenience**: Users wouldn't need external tools
- **Type Safety**: Rust could provide compile-time mapping validation
- **Performance**: Native Rust transformation would be faster than Python

### Arguments AGAINST Implementation
- **Scope Creep**: Transformation is a separate concern from validation
- **Complexity**: Would significantly expand codebase
- **Maintenance**: Another major feature to maintain
- **Alternatives Exist**: Many mature ETL tools available
- **Limited Use**: Not all users need transformation capabilities

### Recommendation
**Do NOT implement linkml-map parity** unless there's specific user demand. Focus on core schema validation, code generation, and production features where RootReal already excels.

If transformation is needed:
1. Create a separate `linkml-transform` crate
2. Keep it independent from core validation
3. Design as optional add-on service
4. Use composition, not integration

## Alternative: Transformation Service Architecture

If transformation features are needed, implement as separate service:

```
┌─────────────────────────────────────────────────────┐
│              LinkML Service (Core)                   │
│  - Schema validation                                 │
│  - Code generation                                   │
│  - Schema management                                 │
└─────────────────────────────────────────────────────┘
                       │
                       │ (optional dependency)
                       ▼
┌─────────────────────────────────────────────────────┐
│         LinkML Transform Service (Optional)          │
│  - Schema-to-schema mapping                          │
│  - Data transformations                              │
│  - ETL pipelines                                     │
└─────────────────────────────────────────────────────┘
```

**Benefits**:
- Core service remains focused
- Users who don't need transformation don't pay for it
- Can be developed/maintained independently
- Clear separation of concerns

## Conclusion

**linkml-map parity is 0% and should remain 0%** unless there's strong user demand. Data transformation is a separate concern best handled by:
1. Custom application code
2. Dedicated ETL tools
3. Optional transformation service (if needed)

RootReal's value proposition is in **high-performance schema validation, production-grade reliability, and comprehensive code generation** - not in data transformation pipelines.

## Related Documentation

- **Core parity evaluation**: `PARITY_EVALUATION.md`
- **Python LinkML**: https://github.com/linkml/linkml
- **linkml-map**: https://github.com/linkml/linkml-map
- **Transformation architecture** (if implemented): TBD
