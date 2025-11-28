#!/usr/bin/env python3
"""
Convert Turtle format with semicolons to N-Triples format
for rust-kgdb compatibility
"""

import re

def expand_prefixes(text):
    """Expand prefix declarations"""
    prefixes = {}
    for match in re.finditer(r'@prefix\s+(\w+):\s+<([^>]+)>\s*\.', text):
        prefixes[match.group(1)] = match.group(2)
    return prefixes

def expand_uri(uri, prefixes):
    """Expand prefixed URI to full URI"""
    if uri.startswith('<') and uri.endswith('>'):
        return uri
    if uri == 'a':
        return '<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>'

    for prefix, namespace in prefixes.items():
        if uri.startswith(prefix + ':'):
            local_part = uri[len(prefix)+1:]
            return f'<{namespace}{local_part}>'

    return uri

def expand_datatype(value, prefixes):
    """Expand datatype in literal"""
    if '^^' in value:
        lit, dtype = value.rsplit('^^', 1)
        dtype_expanded = expand_uri(dtype, prefixes)
        return f'{lit}^^{dtype_expanded}'
    return value

def convert_turtle_to_ntriples(turtle_text):
    """Convert Turtle with semicolons to N-Triples"""
    prefixes = expand_prefixes(turtle_text)

    # Remove prefix declarations
    text = re.sub(r'@prefix[^\n]+\n', '', turtle_text)

    # Split by subject (lines starting with <http...)
    subject_blocks = re.split(r'\n(<http://[^>]+>)', text)

    ntriples = []

    for i in range(1, len(subject_blocks), 2):
        subject = subject_blocks[i]
        if i+1 < len(subject_blocks):
            block = subject_blocks[i+1]

            # Remove indentation and split by semicolon
            lines = block.strip().split(';')

            for line in lines:
                line = line.strip()
                if not line or line == '.':
                    continue

                # Remove trailing period
                line = line.rstrip('.')
                line = line.strip()

                # Split predicate and object
                parts = line.split(None, 1)
                if len(parts) == 2:
                    predicate, obj = parts
                    predicate_expanded = expand_uri(predicate, prefixes)

                    # Check if object is a URI (not a literal)
                    if not obj.startswith('"') and not obj.startswith('<'):
                        obj_expanded = expand_uri(obj, prefixes)
                    else:
                        obj_expanded = expand_datatype(obj, prefixes)

                    ntriples.append(f'{subject} {predicate_expanded} {obj_expanded} .')

    return '\n'.join(ntriples)

# Test data from DEMO (Scenario 1)
scenario1_turtle = """@prefix av: <http://zenya.com/ontology/av#> .
@prefix sensor: <http://zenya.com/ontology/sensor#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

<http://zenya.com/vehicle/ego>
    a av:Vehicle ;
    rdfs:label "Ego Vehicle" ;
    av:hasVelocity "13.3"^^xsd:float ;
    av:positionX "-80.0"^^xsd:float ;
    av:carLength "5.0"^^xsd:float .

<http://zenya.com/traffic_light/tl_001>
    a av:TrafficLight ;
    rdfs:label "Traffic Light 001" ;
    av:state "red"^^xsd:string ;
    av:positionX "-30.0"^^xsd:float ;
    sensor:confidence "0.98"^^xsd:float ;
    av:distanceMeters "30.0"^^xsd:float ."""

print("=== SCENARIO 1: Red Traffic Light ===")
print(convert_turtle_to_ntriples(scenario1_turtle))
print()

# Scenario 2
scenario2_turtle = """@prefix av: <http://zenya.com/ontology/av#> .
@prefix sensor: <http://zenya.com/ontology/sensor#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

<http://zenya.com/vehicle/ego>
    a av:Vehicle ;
    av:hasVelocity "10.0"^^xsd:float ;
    av:positionX "-60.0"^^xsd:float .

<http://zenya.com/crosswalk/cw_001>
    a av:Crosswalk ;
    av:state "active"^^xsd:string ;
    av:positionX "-43.0"^^xsd:float ."""

print("=== SCENARIO 2: Pedestrian Crossing ===")
print(convert_turtle_to_ntriples(scenario2_turtle))
print()

# Scenario 3
scenario3_turtle = """@prefix av: <http://zenya.com/ontology/av#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

<http://zenya.com/vehicle/ego>
    a av:Vehicle ;
    av:hasVelocity "22.2"^^xsd:float ;
    av:positionX "-120.0"^^xsd:float ."""

print("=== SCENARIO 3: School Zone Speeding ===")
print(convert_turtle_to_ntriples(scenario3_turtle))
