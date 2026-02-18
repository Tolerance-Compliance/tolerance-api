# CMMC API

REST API for querying **NIST SP 800-171 Rev 3** security requirements, built for CMMC (Cybersecurity Maturity Model Certification) compliance workflows.

The server loads the official NIST CPRT JSON export at startup, builds a search index for fast lookups, and exposes the data through a clean JSON API. All data is read-only and served from memory -- no database required.

### NEEDED

- The NIST data file (included): `cprt-sp_800_171_3_0_0-20260215-171034.json`

### Run

```bash
cargo run
```

The server starts on `http://0.0.0.0:3000` by default.

### Configuration

Set these environment variables or add them to a `.env` file:

| Variable         | Default                                            | Description                  |
|------------------|----------------------------------------------------|------------------------------|
| `HOST`           | `0.0.0.0`                                          | Bind address                 |
| `PORT`           | `3000`                                              | Bind port                    |
| `NIST_DATA_PATH` | `cprt-sp_800_171_3_0_0-20260215-171034.json`       | Path to the NIST JSON export |
| `RUST_LOG`       | `cmmc_api=info,tower_http=debug`                   | Tracing log filter           |

## API Reference

All endpoints are `GET` and return JSON. The base URL in all examples is `http://localhost:3000`.

### Health Check

```
GET /health
```

```bash
curl http://localhost:3000/health
```

```json
{
  "status": 200,
  "service": "cmmc-api",
  "version": "0.1.0",
  "timestamp": 1739900000000
}
```

---

### Dataset Summary

High-level overview of the loaded NIST 800-171 data, including document metadata and element counts.

```
GET /api/v1/cmmc/summary
```

```bash
curl http://localhost:3000/api/v1/cmmc/summary
```

```json
{
  "document": {
    "doc_identifier": "SP_800_171_3_0_0",
    "name": "SP 800-171",
    "version": "3.0.0",
    "website": "https://csrc.nist.gov/projects/cprt"
  },
  "family_count": 17,
  "requirement_count": 82,
  "security_requirement_count": 176,
  "relationship_count": 512
}
```

---

### Families

Families are the top-level groupings (e.g. Access Control, Audit and Accountability). Each family contains nested requirements and security requirements.

#### List all families

```
GET /api/v1/cmmc/families
```

```bash
curl http://localhost:3000/api/v1/cmmc/families
```

Returns an array of families with their full hierarchy:

```json
[
  {
    "identifier": "03.01",
    "title": "Access Control",
    "requirements": [
      {
        "identifier": "03.01.01",
        "title": "Account Management",
        "text": "a. Define...",
        "security_requirements": [
          {
            "identifier": "SR-03.01.01.a",
            "title": "...",
            "text": "...",
            "discussion": "...",
            "assessment": "..."
          }
        ]
      }
    ]
  }
]
```

#### Get a specific family

```
GET /api/v1/cmmc/families/:id
```

```bash
curl http://localhost:3000/api/v1/cmmc/families/03.01
```

Returns a single family with its nested requirements, or `404` if not found.

---

### Elements

Elements are the raw building blocks of the NIST data: families, requirements, security requirements, discussions, and assessments. The elements endpoint supports **search**, **type filtering**, and **pagination**.

#### List elements (with filtering)

```
GET /api/v1/cmmc/elements
```

| Query Parameter | Type     | Default | Description                                                                       |
|-----------------|----------|---------|-----------------------------------------------------------------------------------|
| `type`          | `string` | --      | Filter by type: `family`, `requirement`, `security_requirement`, `discussion`, `assessment` |
| `search`        | `string` | --      | Search in identifier, title, and text                                             |
| `limit`         | `int`    | `100`   | Results per page (max `1000`)                                                     |
| `offset`        | `int`    | `0`     | Pagination offset                                                                 |

```bash
# All elements (first 100)
curl http://localhost:3000/api/v1/cmmc/elements

# Only families
curl "http://localhost:3000/api/v1/cmmc/elements?type=family"

# Search for "encryption"
curl "http://localhost:3000/api/v1/cmmc/elements?search=encryption"

# Search security requirements for "access"
curl "http://localhost:3000/api/v1/cmmc/elements?type=security_requirement&search=access"

# Page 2 (items 50-99)
curl "http://localhost:3000/api/v1/cmmc/elements?limit=50&offset=50"
```

Response is paginated:

```json
{
  "data": [
    {
      "element_type": "family",
      "element_identifier": "03.01",
      "title": "Access Control",
      "text": "",
      "doc_identifier": "SP_800_171_3_0_0"
    }
  ],
  "total": 420,
  "limit": 100,
  "offset": 0,
  "has_more": true
}
```

#### Get a specific element

```
GET /api/v1/cmmc/elements/:id
```

```bash
curl http://localhost:3000/api/v1/cmmc/elements/03.01.01
```

Returns a single element by its identifier.

---

### Requirements

Convenience endpoint that returns all elements of type `requirement`.

```
GET /api/v1/cmmc/requirements
```

```bash
curl http://localhost:3000/api/v1/cmmc/requirements
```

Returns an array of `Element` objects where `element_type` is `"requirement"`.

---

### Security Requirements

Convenience endpoint that returns all elements of type `security_requirement`.

```
GET /api/v1/cmmc/security-requirements
```

```bash
curl http://localhost:3000/api/v1/cmmc/security-requirements
```

---

### Relationships

Relationships describe how elements connect to each other (e.g. a requirement belonging to a family, or a discussion relating to a security requirement).

#### List all relationships

```
GET /api/v1/cmmc/relationships
```

```bash
curl http://localhost:3000/api/v1/cmmc/relationships
```

```json
[
  {
    "source_element_identifier": "03.01.01",
    "source_doc_identifier": "SP_800_171_3_0_0",
    "dest_element_identifier": "03.01",
    "dest_doc_identifier": "SP_800_171_3_0_0",
    "relationship_identifier": "belongs_to",
    "provenance_doc_identifier": "SP_800_171_3_0_0"
  }
]
```

#### Get relationships for a specific element

```
GET /api/v1/cmmc/elements/:id/relationships
```

```bash
curl http://localhost:3000/api/v1/cmmc/elements/03.01.01/relationships
```

Returns all relationships where the element appears as either source or destination.

---

## Data Model

The API exposes NIST SP 800-171 data in a hierarchy:

```
Family (e.g. "03.01 Access Control")
  └── Requirement (e.g. "03.01.01 Account Management")
        └── Security Requirement (e.g. "SR-03.01.01.a")
              ├── Discussion (optional)
              └── Assessment (optional)
```

### Element Types

| Type                     | Description                                      |
|--------------------------|--------------------------------------------------|
| `family`                 | Top-level grouping (e.g. Access Control)          |
| `requirement`            | Specific requirement within a family              |
| `security_requirement`   | Detailed security requirement under a requirement |
| `discussion`             | Supplementary discussion text                     |
| `assessment`             | Assessment procedure text                         |

### Identifier Format

Identifiers follow a hierarchical dot notation:

- Family: `03.01`
- Requirement: `03.01.01`
- Security Requirement: `SR-03.01.01.a`

---

## Error Responses

Errors return JSON with an `error` message and `success: false`:

```json
{
  "error": "Family '99.99' not found",
  "success": false
}
```

| Status | Meaning           |
|--------|-------------------|
| `404`  | Element not found |
| `400`  | Bad request       |
