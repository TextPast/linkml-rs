# TextPast/RootReal LinkML Conventions

## Overview

RootReal uses enhanced LinkML conventions that extend the standard LinkML specification with TextPast-specific features for schema and instance management, import resolution, and metadata handling.

## Schema Location

### Current Location (v2.0+)
All production LinkML schemas are located in:
```
crates/model/symbolic/schemata/
```

### Legacy Location (Deprecated)
```
domain/schema/  # No longer used
```

### Directory Structure
```
crates/model/symbolic/schemata/
├── meta/                    # Meta-level schemas
│   ├── entity/
│   │   └── hyperentity/
│   │       └── schema.yaml
│   ├── identifier/
│   │   ├── identifier/
│   │   │   └── schema.yaml
│   │   ├── curie/
│   │   └── fqn/
│   ├── label/
│   ├── description/
│   └── ...
└── place/                   # Geographic entities
    └── polity/
        └── country/
            ├── schema.yaml
            └── iso_3166_entity.yaml
```

## Schema File Conventions

### Schema File Structure
Schema files follow the pattern: `{domain}/{subdomain}/schema.yaml`

**Example**: `crates/model/symbolic/schemata/place/polity/country/schema.yaml`

```yaml
id: https://textpast.org/schema/place/polity/country
name: country
version: 1.0.0
created_on: '2025-01-22T16:39:26+01:00'
last_updated_on: '2025-01-22T16:39:26+01:00'
description: 'Geopolitical entities officially recognised by the international community.'
prefixes:
  txp: https://textpast.org/
  linkml: https://w3id.org/linkml/
default_prefix: txp
default_range: string
imports:
  - linkml:types
  - txp:meta/entity/hyperentity/schema
  - txp:meta/label/label/schema
classes:
  ISO3166Entity:
    is_a: Entity
    description: 'A geopolitical entity as defined in ISO 3166-1'
    slots:
      - identifier
      - label
      - tld
```

### Required Metadata Fields
All schema files MUST include:
- `id`: Full HTTPS URL (e.g., `https://textpast.org/schema/place/polity/country`)
- `name`: Short name (e.g., `country`)
- `version`: Semantic version (e.g., `1.0.0`)
- `created_on`: ISO 8601 timestamp
- `last_updated_on`: ISO 8601 timestamp

## Instance File Conventions

### Instance File Structure
Instance files explicitly reference their schema and contain real metadata (not commented out).

**Example**: `crates/model/symbolic/schemata/place/polity/country/iso_3166_entity.yaml`

```yaml
id: https://textpast.org/instance/place/polity/country/iso_3166_entity
schema: https://textpast.org/schema/place/polity/country/schema
name: iso_3166_entity
title: ISO 3166 Entity
version: 1.0.0
created_on: "2025-03-30T10:41:26+01:00"
last_updated_on: "2025-03-07T16:39:26+01:00"
instances:
  - id: "US"
    label: "United States of America"
    tld: ".us"
    exact_mappings:
      - wd:Q30
    notes: "Previous ISO country name: United States."
  
  - id: "GB"
    label: "United Kingdom of Great Britain and Northern Ireland"
    tld: ".gb"
    exact_mappings:
      - wd:Q145
    notes: ".uk is the primary ccTLD instead of .gb."
```

### Required Instance Metadata
All instance files MUST include:
- `id`: Full HTTPS URL for the instance collection
- `schema`: Full HTTPS URL of the schema this conforms to
- `name`: Short name
- `version`: Semantic version
- `created_on`: ISO 8601 timestamp
- `last_updated_on`: ISO 8601 timestamp
- `instances`: Array of instance objects (NOT commented out)

## Import Resolution with txp: Prefix

### Overview
The `txp:` prefix provides intelligent import resolution with local-first caching and remote fallback.

### Resolution Strategy
1. **Local Cache Check**: Check RootReal's Cache Service for previously fetched schemas
2. **Local File System**: Check `crates/model/symbolic/schemata/`
3. **Remote Fetch**: Fetch from `https://textpast.org/` and cache locally

### Path Mapping Rules

#### Schema Imports
Format: `txp:{domain}/{subdomain}/schema`

**Example**: `txp:meta/entity/hyperentity/schema`
- Local path: `crates/model/symbolic/schemata/meta/entity/hyperentity/schema.yaml`
- Remote URL: `https://textpast.org/schema/meta/entity/hyperentity`
- Note: `/schema` suffix is removed from URL

#### Instance Imports
Format: `txp:{domain}/{subdomain}/{entity_name}`

**Example**: `txp:place/polity/country/iso_3166_entity`
- Local path: `crates/model/symbolic/schemata/place/polity/country/iso_3166_entity.yaml`
- Remote URL: `https://textpast.org/instance/place/polity/country/iso_3166_entity`
- Note: Instance URLs use `/instance/` instead of `/schema/`

### Caching Behavior
- **Cache Service**: All fetched schemas are cached using RootReal's Cache Service
- **TTL**: Default 1 hour (configurable)
- **Invalidation**: Manual or on schema version change
- **Performance**: Local cache hits are ~100x faster than remote fetches

## Validation and Type Checking

### Automatic Validation
The LinkML service automatically validates:
1. **Schema structure**: Conforms to LinkML metamodel
2. **Instance conformance**: Instances match their declared schema
3. **Type constraints**: All type-specific validations (patterns, ranges, etc.)
4. **Cross-references**: Valid CURIEs and mappings

### Scoped Import Resolution for Slot Ranges

**New Convention**: You can optionally specify which imports to search for slot range types.

This is useful when multiple imports define classes with the same name, allowing you to disambiguate which class should be used for a specific slot.

```yaml
# In schema.yaml
imports:
  - linkml:types
  - txp:meta/entity/hyperentity/schema
  - txp:meta/label/label/schema
  - txp:meta/identifier/identifier/schema

classes:
  ISO3166Entity:
    slot_usage:
      identifier:
        range: CountryCodeAlpha2Identifier
        imports:
          - txp:meta/identifier/identifier/schema  # Only search this import
        required: true
      tld:
        required: true  # No imports specified, search all imports
```

**Resolution Rules**:
- **With `imports` specified**: Only search the specified imports for the range type
- **Without `imports`**: Search all schema imports for the range type
- **Not found**: Return validation error indicating the type cannot be resolved

### Example: ISO 3166 Country Code Validation
```yaml
# In schema.yaml
classes:
  ISO3166Entity:
    slot_usage:
      identifier:
        range: CountryCodeAlpha2Identifier
        imports:
          - txp:meta/identifier/identifier/schema
        required: true
```

The `CountryCodeAlpha2Identifier` type automatically enforces:
- Exactly 2 uppercase letters
- Valid ISO 3166-1 alpha-2 code
- Pattern: `^[A-Z]{2}$`

## Integration with External API Service

For remote schema fetching, the LinkML service integrates with:
- **Service**: `crates/hub/api/integration/external`
- **Purpose**: HTTP requests to `https://textpast.org/`
- **Features**: Rate limiting, retry logic, error handling
- **Caching**: Automatic caching via Cache Service

## Best Practices

1. **Always use txp: imports** for TextPast schemas
2. **Include full metadata** in both schemas and instances
3. **Version your schemas** using semantic versioning
4. **Test locally first** before relying on remote fetches
5. **Use Cache Service** to minimize remote requests
6. **Document schema changes** in version history

## Migration from Legacy Location

If you have schemas in `domain/schema/`, migrate them to `crates/model/symbolic/schemata/`:

```bash
# Example migration
mv domain/schema/place/polity/country/schema.yaml \
   crates/model/symbolic/schemata/place/polity/country/schema.yaml
```

Update all `txp:` imports - they will automatically resolve to the new location.

