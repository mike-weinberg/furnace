# JSON Schema Inference

Testing implementation plan quality by building a JSON schema inference function from scratch.

## Structure

```
schema_inference/
├── src/
│   ├── lib/
│   │   └── infer_schema.py
│   ├── tests/
│   │   └── examples/
│   │       └── <api_name>/
│   │           └── <endpoint_name>.json
│   └── benchmarking/
└── requirements.txt
```

## Setup

```bash
python3 -m venv venv
source venv/bin/activate
pip install -r requirements.txt
```
