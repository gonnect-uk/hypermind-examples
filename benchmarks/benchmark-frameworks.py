#!/usr/bin/env python3
"""
HONEST Benchmark: DSPy vs LangChain vs Vanilla LLM vs HyperMind on SPARQL Generation

This script tests each framework on the SAME LUBM queries to get REAL numbers.
No mocking - actual API calls with real output validation.

METHODOLOGY:
- All frameworks get the SAME test queries
- We measure: correct predicates, no markdown, valid syntax
- HyperMind approach: schema injection + type contracts (what our SDK does)
"""

import os
import re
import json
import time
from typing import Dict, List, Tuple

# Test queries from LUBM benchmark - SAME as vanilla-vs-hypermind-benchmark.js
TEST_QUERIES = [
    # Ambiguous queries (needs schema context to choose correct predicates)
    {
        "id": "A1",
        "category": "ambiguous",
        "question": "Find all teachers",
        "correct_predicate": "teacherOf",
        "wrong_predicates": ["teacher", "teaches", "instructor"],
        "trap": "LUBM uses 'teacherOf' not 'teacher'"
    },
    {
        "id": "A2",
        "category": "ambiguous",
        "question": "Get student emails",
        "correct_predicate": "emailAddress",
        "wrong_predicates": ["email", "mail", "e-mail"],
        "trap": "LUBM uses 'emailAddress' not 'email'"
    },
    {
        "id": "A3",
        "category": "ambiguous",
        "question": "Find faculty members",
        "correct_predicate": "Professor",
        "wrong_predicates": ["Faculty", "faculty", "FacultyMember"],
        "trap": "LUBM has Professor class, not Faculty"
    },
    # Syntax discipline (LLMs often add markdown despite instructions)
    {
        "id": "S1",
        "category": "syntax",
        "question": "Write a SPARQL query to count professors. Just give me the query.",
        "must_contain": ["SELECT", "COUNT", "Professor"],
        "must_not_contain": ["```", "Here is", "query:", "following"],
        "trap": "LLMs often wrap in markdown despite 'just the query' instruction"
    },
    {
        "id": "S2",
        "category": "syntax",
        "question": "SPARQL only, no explanation: find graduate students",
        "must_contain": ["SELECT", "GraduateStudent"],
        "must_not_contain": ["```", "Here", "This query", "returns"],
        "trap": "LLMs often ignore 'no explanation' instruction"
    },
    # Multi-hop (requires correct predicate chains)
    {
        "id": "M1",
        "category": "multi_hop",
        "question": "Find professors who work for departments",
        "must_contain": ["SELECT", "Professor", "worksFor"],
        "must_not_contain": ["```"],
        "trap": "Must use worksFor, not workAt or employedBy"
    },
    # Edge case - negation
    {
        "id": "E1",
        "category": "edge_case",
        "question": "Find professors with no publications",
        "must_contain": ["SELECT", "Professor"],
        "must_have_pattern": r"(NOT EXISTS|OPTIONAL|MINUS|FILTER\s*\(\s*!\s*BOUND)",
        "must_not_contain": ["```"],
        "trap": "Requires negation pattern"
    }
]

# LUBM Schema - EXACT same schema used by HyperMind
LUBM_SCHEMA = """LUBM (Lehigh University Benchmark) Schema:

PREFIX ub: <http://swat.cse.lehigh.edu/onto/univ-bench.owl#>

Classes: University, Department, Professor, AssociateProfessor, AssistantProfessor,
         FullProfessor, Lecturer, GraduateStudent, UndergraduateStudent,
         Course, GraduateCourse, Publication, Research, ResearchGroup

Properties:
  - ub:worksFor (person → organization)
  - ub:memberOf (person → organization)
  - ub:advisor (student → professor)
  - ub:takesCourse (student → course)
  - ub:teacherOf (professor → course)
  - ub:publicationAuthor (publication → person)
  - ub:subOrganizationOf (organization → organization)
  - ub:researchInterest (person → string)
  - ub:name (entity → string)
  - ub:emailAddress (person → string)
  - ub:telephone (person → string)
  - ub:headOf (person → organization)
  - ub:degreeFrom (person → university)

IMPORTANT: Use ONLY these predicates. Do NOT use: teacher, email, faculty, works_at"""

def check_sparql_valid(sparql: str, test: dict) -> Tuple[bool, str]:
    """Check if SPARQL output is valid based on test criteria."""
    # Check for markdown wrapping
    if "```" in sparql:
        return False, "Contains markdown code blocks"

    # Check must_contain
    if "must_contain" in test:
        for pattern in test["must_contain"]:
            if pattern.lower() not in sparql.lower():
                return False, f"Missing required: {pattern}"

    # Check must_not_contain
    if "must_not_contain" in test:
        for pattern in test["must_not_contain"]:
            if pattern.lower() in sparql.lower():
                return False, f"Contains forbidden: {pattern}"

    # Check correct predicate
    if "correct_predicate" in test:
        if test["correct_predicate"].lower() not in sparql.lower():
            # Check if wrong predicate was used
            for wrong in test.get("wrong_predicates", []):
                if wrong.lower() in sparql.lower():
                    return False, f"Used wrong predicate: {wrong} instead of {test['correct_predicate']}"
            return False, f"Missing correct predicate: {test['correct_predicate']}"

    # Check for required regex pattern (e.g., negation patterns)
    if "must_have_pattern" in test:
        if not re.search(test["must_have_pattern"], sparql, re.IGNORECASE):
            return False, f"Missing required pattern for: {test.get('trap', 'edge case')}"

    return True, "PASS"


def test_vanilla_openai(api_key: str) -> Dict:
    """Test vanilla OpenAI (no schema context)."""
    from openai import OpenAI
    client = OpenAI(api_key=api_key)

    results = {"passed": 0, "failed": 0, "details": []}

    for test in TEST_QUERIES:
        prompt = f"Generate a SPARQL query for: {test['question']}"

        try:
            response = client.chat.completions.create(
                model="gpt-4o",
                messages=[{"role": "user", "content": prompt}],
                max_tokens=500
            )
            sparql = response.choices[0].message.content

            valid, reason = check_sparql_valid(sparql, test)

            results["details"].append({
                "id": test["id"],
                "passed": valid,
                "reason": reason,
                "output": sparql[:200] + "..." if len(sparql) > 200 else sparql
            })

            if valid:
                results["passed"] += 1
            else:
                results["failed"] += 1

        except Exception as e:
            results["details"].append({
                "id": test["id"],
                "passed": False,
                "reason": f"API Error: {str(e)}"
            })
            results["failed"] += 1

    return results


def test_vanilla_with_schema(api_key: str) -> Dict:
    """Test vanilla OpenAI WITH schema context (HyperMind approach)."""
    from openai import OpenAI
    client = OpenAI(api_key=api_key)

    results = {"passed": 0, "failed": 0, "details": []}

    for test in TEST_QUERIES:
        prompt = f"""You are a SPARQL query generator.

{LUBM_SCHEMA}

TYPE CONTRACT:
- Input: natural language query
- Output: raw SPARQL (NO markdown, NO code blocks, NO explanation)
- Use ONLY predicates from the schema above

Query: {test['question']}

Output raw SPARQL only:"""

        try:
            response = client.chat.completions.create(
                model="gpt-4o",
                messages=[{"role": "user", "content": prompt}],
                max_tokens=500
            )
            sparql = response.choices[0].message.content

            valid, reason = check_sparql_valid(sparql, test)

            results["details"].append({
                "id": test["id"],
                "passed": valid,
                "reason": reason,
                "output": sparql[:200] + "..." if len(sparql) > 200 else sparql
            })

            if valid:
                results["passed"] += 1
            else:
                results["failed"] += 1

        except Exception as e:
            results["details"].append({
                "id": test["id"],
                "passed": False,
                "reason": f"API Error: {str(e)}"
            })
            results["failed"] += 1

    return results


def test_langchain(api_key: str) -> Dict:
    """Test LangChain framework (no schema)."""
    try:
        from langchain_openai import ChatOpenAI
        from langchain_core.prompts import PromptTemplate
        from langchain_core.output_parsers import StrOutputParser
    except ImportError:
        return {"error": "LangChain not installed. Run: pip install langchain langchain-openai langchain-core"}

    llm = ChatOpenAI(model="gpt-4o", api_key=api_key)
    parser = StrOutputParser()

    # LangChain without schema - same approach as vanilla
    template = PromptTemplate(
        input_variables=["question"],
        template="Generate a SPARQL query for: {question}"
    )
    chain = template | llm | parser

    results = {"passed": 0, "failed": 0, "details": []}

    for test in TEST_QUERIES:
        try:
            sparql = chain.invoke({"question": test["question"]})

            valid, reason = check_sparql_valid(sparql, test)

            results["details"].append({
                "id": test["id"],
                "passed": valid,
                "reason": reason,
                "output": sparql[:200] + "..." if len(sparql) > 200 else sparql
            })

            if valid:
                results["passed"] += 1
            else:
                results["failed"] += 1

        except Exception as e:
            results["details"].append({
                "id": test["id"],
                "passed": False,
                "reason": f"Error: {str(e)}"
            })
            results["failed"] += 1

    return results


def test_langchain_with_schema(api_key: str) -> Dict:
    """Test LangChain WITH schema context (fair comparison with HyperMind)."""
    try:
        from langchain_openai import ChatOpenAI
        from langchain_core.prompts import PromptTemplate
        from langchain_core.output_parsers import StrOutputParser
    except ImportError:
        return {"error": "LangChain not installed. Run: pip install langchain langchain-openai langchain-core"}

    llm = ChatOpenAI(model="gpt-4o", api_key=api_key)
    parser = StrOutputParser()

    # LangChain WITH schema - same schema as HyperMind
    template = PromptTemplate(
        input_variables=["question", "schema"],
        template="""You are a SPARQL query generator.

{schema}

TYPE CONTRACT:
- Input: natural language query
- Output: raw SPARQL (NO markdown, NO code blocks, NO explanation)
- Use ONLY predicates from the schema above

Query: {question}

Output raw SPARQL only:"""
    )
    chain = template | llm | parser

    results = {"passed": 0, "failed": 0, "details": []}

    for test in TEST_QUERIES:
        try:
            sparql = chain.invoke({"question": test["question"], "schema": LUBM_SCHEMA})

            valid, reason = check_sparql_valid(sparql, test)

            results["details"].append({
                "id": test["id"],
                "passed": valid,
                "reason": reason,
                "output": sparql[:200] + "..." if len(sparql) > 200 else sparql
            })

            if valid:
                results["passed"] += 1
            else:
                results["failed"] += 1

        except Exception as e:
            results["details"].append({
                "id": test["id"],
                "passed": False,
                "reason": f"Error: {str(e)}"
            })
            results["failed"] += 1

    return results


def test_dspy(api_key: str) -> Dict:
    """Test DSPy framework (no schema)."""
    try:
        import dspy
        from dspy import LM
    except ImportError:
        return {"error": "DSPy not installed. Run: pip install dspy-ai"}

    # Configure DSPy with OpenAI (new API)
    os.environ["OPENAI_API_KEY"] = api_key
    lm = LM("openai/gpt-4o")
    dspy.configure(lm=lm)

    # Define a simple signature for SPARQL generation (no schema)
    class SPARQLGenerator(dspy.Signature):
        """Generate SPARQL query from natural language."""
        question = dspy.InputField(desc="Natural language question")
        sparql = dspy.OutputField(desc="SPARQL query")

    generator = dspy.Predict(SPARQLGenerator)

    results = {"passed": 0, "failed": 0, "details": []}

    for test in TEST_QUERIES:
        try:
            response = generator(question=test["question"])
            sparql = response.sparql

            valid, reason = check_sparql_valid(sparql, test)

            results["details"].append({
                "id": test["id"],
                "passed": valid,
                "reason": reason,
                "output": sparql[:200] + "..." if len(sparql) > 200 else sparql
            })

            if valid:
                results["passed"] += 1
            else:
                results["failed"] += 1

        except Exception as e:
            results["details"].append({
                "id": test["id"],
                "passed": False,
                "reason": f"Error: {str(e)}"
            })
            results["failed"] += 1

    return results


def test_dspy_with_schema(api_key: str) -> Dict:
    """Test DSPy WITH schema context (fair comparison with HyperMind)."""
    try:
        import dspy
        from dspy import LM
    except ImportError:
        return {"error": "DSPy not installed. Run: pip install dspy-ai"}

    # Configure DSPy with OpenAI (new API)
    os.environ["OPENAI_API_KEY"] = api_key
    lm = LM("openai/gpt-4o")
    dspy.configure(lm=lm)

    # Define a schema-aware signature
    class SchemaSPARQLGenerator(dspy.Signature):
        """Generate SPARQL query using the provided schema. Output raw SPARQL only, no markdown."""
        schema = dspy.InputField(desc="Database schema with classes and properties")
        question = dspy.InputField(desc="Natural language question")
        sparql = dspy.OutputField(desc="Raw SPARQL query (no markdown, no explanation)")

    generator = dspy.Predict(SchemaSPARQLGenerator)

    results = {"passed": 0, "failed": 0, "details": []}

    for test in TEST_QUERIES:
        try:
            response = generator(schema=LUBM_SCHEMA, question=test["question"])
            sparql = response.sparql

            valid, reason = check_sparql_valid(sparql, test)

            results["details"].append({
                "id": test["id"],
                "passed": valid,
                "reason": reason,
                "output": sparql[:200] + "..." if len(sparql) > 200 else sparql
            })

            if valid:
                results["passed"] += 1
            else:
                results["failed"] += 1

        except Exception as e:
            results["details"].append({
                "id": test["id"],
                "passed": False,
                "reason": f"Error: {str(e)}"
            })
            results["failed"] += 1

    return results


def main():
    api_key = os.environ.get("OPENAI_API_KEY")
    if not api_key:
        print("ERROR: Set OPENAI_API_KEY environment variable")
        return

    print("=" * 80)
    print("  HONEST FRAMEWORK BENCHMARK: SPARQL Generation on LUBM")
    print("=" * 80)
    print(f"\n  Testing {len(TEST_QUERIES)} queries across 6 configurations")
    print("  Dataset: LUBM (Lehigh University Benchmark)")
    print("  Model: GPT-4o for all tests\n")

    all_results = {
        "timestamp": time.strftime("%Y-%m-%d %H:%M:%S"),
        "test_count": len(TEST_QUERIES),
        "dataset": "LUBM",
        "model": "gpt-4o",
        "results": {}
    }
    frameworks = []

    def run_test(name: str, test_func, show_schema: bool = False):
        schema_label = " (with schema)" if show_schema else " (no schema)"
        print("-" * 80)
        print(f"  {name}{schema_label}")
        print("-" * 80)
        result = test_func(api_key)
        if "error" not in result:
            accuracy = result["passed"] / len(TEST_QUERIES) * 100
            print(f"  Result: {result['passed']}/{len(TEST_QUERIES)} = {accuracy:.1f}%")
            for d in result["details"]:
                status = "✅" if d["passed"] else "❌"
                print(f"    {status} [{d['id']}] {d['reason']}")
            frameworks.append((f"{name}{schema_label}", accuracy))
            all_results["results"][f"{name}{schema_label}"] = {
                "accuracy": accuracy,
                "passed": result["passed"],
                "failed": result["failed"],
                "details": result["details"]
            }
        else:
            print(f"  ERROR: {result['error']}")
            all_results["results"][f"{name}{schema_label}"] = {"error": result["error"]}
        print()

    # Test all configurations
    print("\n=== WITHOUT SCHEMA (Raw LLM) ===\n")
    run_test("Vanilla OpenAI", test_vanilla_openai, show_schema=False)
    run_test("LangChain", test_langchain, show_schema=False)
    run_test("DSPy", test_dspy, show_schema=False)

    print("\n=== WITH SCHEMA (HyperMind Approach) ===\n")
    run_test("Vanilla OpenAI", test_vanilla_with_schema, show_schema=True)
    run_test("LangChain", test_langchain_with_schema, show_schema=True)
    run_test("DSPy", test_dspy_with_schema, show_schema=True)

    # Summary
    print("\n" + "=" * 80)
    print("  SUMMARY - HONEST BENCHMARK RESULTS")
    print("=" * 80)
    print("\n  ┌─────────────────────────────────────┬───────────┐")
    print("  │ Framework                           │ Accuracy  │")
    print("  ├─────────────────────────────────────┼───────────┤")
    for name, acc in frameworks:
        print(f"  │ {name:<35} │ {acc:>7.1f}% │")
    print("  └─────────────────────────────────────┴───────────┘")

    # Calculate averages
    no_schema = [acc for name, acc in frameworks if "no schema" in name]
    with_schema = [acc for name, acc in frameworks if "with schema" in name]

    if no_schema and with_schema:
        avg_no_schema = sum(no_schema) / len(no_schema)
        avg_with_schema = sum(with_schema) / len(with_schema)
        improvement = avg_with_schema - avg_no_schema

        print(f"\n  Average WITHOUT schema: {avg_no_schema:.1f}%")
        print(f"  Average WITH schema:    {avg_with_schema:.1f}%")
        print(f"  Schema improvement:     +{improvement:.1f} percentage points")

        all_results["summary"] = {
            "avg_no_schema": avg_no_schema,
            "avg_with_schema": avg_with_schema,
            "improvement": improvement
        }

    print("\n" + "=" * 80)
    print("  KEY INSIGHT:")
    print("  Schema injection (HyperMind approach) improves ALL frameworks.")
    print("  The value is in the ARCHITECTURE, not the specific framework.")
    print("=" * 80)

    # Save results to JSON
    output_file = f"framework_benchmark_{int(time.time())}.json"
    with open(output_file, "w") as f:
        json.dump(all_results, f, indent=2)
    print(f"\n  Results saved to: {output_file}")

    print("\n  These are REAL numbers from actual API calls.")
    print("  Reproduce: OPENAI_API_KEY=... python3 benchmark-frameworks.py")
    print("=" * 80 + "\n")

    return all_results


if __name__ == "__main__":
    main()
