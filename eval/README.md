# Evaluation & Benchmarking

This directory contains evaluation datasets and benchmarking tools for AiMitra.

## Datasets

Datasets are JSON files containing evaluation prompts and expected outputs.

### Format

```json
{
  "name": "dataset-name",
  "version": "1.0.0",
  "description": "Description of the dataset",
  "tests": [
    {
      "category": "code-generation",
      "prompt": "Write a Python function to...",
      "expected": {
        "contains": ["def ", "return"]
      },
      "difficulty": "easy"
    }
  ]
}
```

## Running Evaluations

```bash
# Run a specific dataset
cargo run --bin aimitra-eval -- dataset.json

# Run all benchmarks
cargo run --bin aimitra-bench

# Generate report
cargo run --bin aimitra-report
```

## Available Datasets

| Dataset | Size | Focus |
|---------|------|-------|
| code-generation.json | 100 tests | Code generation |
| code-completion.json | 200 tests | Code completion |
| reasoning.json | 50 tests | Reasoning tasks |
| math.json | 75 tests | Mathematical reasoning |

## Metrics

Evaluations measure:
- **Accuracy**: Correctness of output
- **Speed**: Tokens per second
- **Memory**: RAM usage during inference
- **Latency**: Time to first token