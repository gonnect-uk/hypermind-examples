// CSV to Turtle (TTL) Converter
// Converts real-world datasets to RDF format for mobile graph database
// Usage: rustc -O csv_to_ttl.rs && ./csv_to_ttl input.csv output.ttl --schema order --limit 10000

use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::collections::HashMap;

#[derive(Debug, Clone)]
enum SchemaType {
    Order,        // schema.org Order/Product/Customer
    Insurance,    // Custom insurance vocabulary
    SupplyChain,  // Custom supply chain vocabulary
}

struct TTLConverter {
    schema_type: SchemaType,
    base_uri: String,
    limit: Option<usize>,
}

impl TTLConverter {
    fn new(schema_type: SchemaType, limit: Option<usize>) -> Self {
        let base_uri = match schema_type {
            SchemaType::Order => "http://example.org/data/".to_string(),
            SchemaType::Insurance => "http://example.org/insurance/".to_string(),
            SchemaType::SupplyChain => "http://example.org/supplychain/".to_string(),
        };

        Self {
            schema_type,
            base_uri,
            limit,
        }
    }

    fn generate_prefixes(&self) -> String {
        match self.schema_type {
            SchemaType::Order => r#"@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix schema: <https://schema.org/> .
@prefix : <http://example.org/data/> .

"#.to_string(),
            SchemaType::Insurance => r#"@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix schema: <https://schema.org/> .
@prefix ins: <http://example.org/insurance/> .
@prefix : <http://example.org/insurance/> .

"#.to_string(),
            SchemaType::SupplyChain => r#"@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix schema: <https://schema.org/> .
@prefix sc: <http://example.org/supplychain/> .
@prefix : <http://example.org/supplychain/> .

"#.to_string(),
        }
    }

    fn sanitize_uri(&self, value: &str) -> String {
        value
            .chars()
            .map(|c| match c {
                ' ' => '_',
                '/' | '\\' | '?' | '#' | '[' | ']' | '@' | '!' | '$' | '&' | '\'' | '(' | ')' | '*' | '+' | ',' | ';' | '=' => '_',
                _ if c.is_alphanumeric() || c == '_' || c == '-' || c == '.' => c,
                _ => '_',
            })
            .collect()
    }

    fn escape_literal(&self, value: &str) -> String {
        value
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t")
    }

    fn convert_csv_to_ttl(&self, input_path: &str, output_path: &str) -> std::io::Result<()> {
        let input_file = File::open(input_path)?;
        let reader = BufReader::new(input_file);
        let mut lines = reader.lines();

        // Read header
        let header = lines.next().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "Empty CSV file")
        })??;

        let columns: Vec<String> = header.split(',')
            .map(|s| s.trim().trim_matches('"').to_string())
            .collect();

        println!("📊 CSV Columns: {:?}", columns);
        println!("🔧 Schema Type: {:?}", self.schema_type);

        // Create output file
        let mut output_file = File::create(output_path)?;

        // Write prefixes
        output_file.write_all(self.generate_prefixes().as_bytes())?;
        output_file.write_all(b"# Generated from CSV data\n\n")?;

        // Process rows
        let mut row_count = 0;
        let mut triple_count = 0;

        for line in lines {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            // Check limit
            if let Some(limit) = self.limit {
                if row_count >= limit {
                    break;
                }
            }

            let values: Vec<String> = self.parse_csv_line(&line);

            if values.len() != columns.len() {
                eprintln!("⚠️  Skipping row {} (column count mismatch)", row_count + 1);
                continue;
            }

            let row_data: HashMap<String, String> = columns.iter()
                .zip(values.iter())
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();

            let triples = self.generate_triples(&row_data, row_count);
            triple_count += triples.len();

            for triple in triples {
                output_file.write_all(triple.as_bytes())?;
                output_file.write_all(b"\n")?;
            }
            output_file.write_all(b"\n")?;

            row_count += 1;
            if row_count % 1000 == 0 {
                println!("   Processed {} rows, {} triples...", row_count, triple_count);
            }
        }

        println!("✅ Conversion complete!");
        println!("   📈 Rows: {}", row_count);
        println!("   🔗 Triples: {}", triple_count);
        println!("   📁 Output: {}", output_path);

        Ok(())
    }

    fn parse_csv_line(&self, line: &str) -> Vec<String> {
        let mut values = Vec::new();
        let mut current = String::new();
        let mut in_quotes = false;
        let mut chars = line.chars().peekable();

        while let Some(c) = chars.next() {
            match c {
                '"' => {
                    if in_quotes && chars.peek() == Some(&'"') {
                        // Escaped quote
                        current.push('"');
                        chars.next();
                    } else {
                        in_quotes = !in_quotes;
                    }
                }
                ',' if !in_quotes => {
                    values.push(current.trim().to_string());
                    current.clear();
                }
                _ => current.push(c),
            }
        }
        values.push(current.trim().to_string());
        values
    }

    fn generate_triples(&self, row: &HashMap<String, String>, row_id: usize) -> Vec<String> {
        match self.schema_type {
            SchemaType::Order => self.generate_order_triples(row, row_id),
            SchemaType::Insurance => self.generate_insurance_triples(row, row_id),
            SchemaType::SupplyChain => self.generate_supplychain_triples(row, row_id),
        }
    }

    fn generate_order_triples(&self, row: &HashMap<String, String>, row_id: usize) -> Vec<String> {
        let mut triples = Vec::new();
        let order_id = format!("Order_{}", row_id);

        // Order entity
        triples.push(format!(":{}  rdf:type  schema:Order .", order_id));

        // Map common CSV columns to schema.org properties
        if let Some(order_num) = row.get("Order Id") {
            triples.push(format!(":{}  schema:orderNumber  \"{}\" .", order_id, self.escape_literal(order_num)));
        }

        if let Some(date) = row.get("order date (DateOrders)") {
            if !date.is_empty() {
                triples.push(format!(":{}  schema:orderDate  \"{}\"^^xsd:date .", order_id, self.escape_literal(date)));
            }
        }

        if let Some(status) = row.get("Order Status") {
            if !status.is_empty() {
                triples.push(format!(":{}  schema:orderStatus  \"{}\" .", order_id, self.escape_literal(status)));
            }
        }

        // Customer
        if let Some(customer_id) = row.get("Customer Id") {
            let customer_uri = format!("Customer_{}", self.sanitize_uri(customer_id));
            triples.push(format!(":{}  schema:customer  :{} .", order_id, customer_uri));
            triples.push(format!(":{}  rdf:type  schema:Person .", customer_uri));

            if let Some(name) = row.get("Customer Fname") {
                if !name.is_empty() {
                    triples.push(format!(":{}  schema:givenName  \"{}\" .", customer_uri, self.escape_literal(name)));
                }
            }

            if let Some(lname) = row.get("Customer Lname") {
                if !lname.is_empty() {
                    triples.push(format!(":{}  schema:familyName  \"{}\" .", customer_uri, self.escape_literal(lname)));
                }
            }

            if let Some(city) = row.get("Customer City") {
                if !city.is_empty() {
                    triples.push(format!(":{}  schema:addressLocality  \"{}\" .", customer_uri, self.escape_literal(city)));
                }
            }

            if let Some(country) = row.get("Customer Country") {
                if !country.is_empty() {
                    triples.push(format!(":{}  schema:addressCountry  \"{}\" .", customer_uri, self.escape_literal(country)));
                }
            }

            if let Some(segment) = row.get("Customer Segment") {
                if !segment.is_empty() {
                    triples.push(format!(":{}  schema:jobTitle  \"{}\" .", customer_uri, self.escape_literal(segment)));
                }
            }
        }

        // Product
        if let Some(product_name) = row.get("Product Name") {
            let product_id = format!("Product_{}", self.sanitize_uri(product_name));
            triples.push(format!(":{}  schema:orderedItem  :{} .", order_id, product_id));
            triples.push(format!(":{}  rdf:type  schema:Product .", product_id));
            triples.push(format!(":{}  schema:name  \"{}\" .", product_id, self.escape_literal(product_name)));

            if let Some(category) = row.get("Category Name") {
                if !category.is_empty() {
                    triples.push(format!(":{}  schema:category  \"{}\" .", product_id, self.escape_literal(category)));
                }
            }

            if let Some(price) = row.get("Product Price") {
                if let Ok(price_val) = price.parse::<f64>() {
                    triples.push(format!(":{}  schema:price  \"{}\"^^xsd:decimal .", product_id, price_val));
                }
            }
        }

        // Sales info
        if let Some(sales) = row.get("Sales") {
            if let Ok(sales_val) = sales.parse::<f64>() {
                triples.push(format!(":{}  schema:totalPrice  \"{}\"^^xsd:decimal .", order_id, sales_val));
            }
        }

        if let Some(quantity) = row.get("Order Item Quantity") {
            if let Ok(qty) = quantity.parse::<i32>() {
                triples.push(format!(":{}  schema:orderQuantity  \"{}\"^^xsd:integer .", order_id, qty));
            }
        }

        triples
    }

    fn generate_insurance_triples(&self, row: &HashMap<String, String>, row_id: usize) -> Vec<String> {
        let mut triples = Vec::new();
        let policy_id = format!("Policy_{}", row_id);

        triples.push(format!(":{}  rdf:type  ins:InsurancePolicy .", policy_id));

        // Add insurance-specific mappings here
        // This is a template - actual fields depend on the dataset

        triples
    }

    fn generate_supplychain_triples(&self, row: &HashMap<String, String>, row_id: usize) -> Vec<String> {
        let mut triples = Vec::new();
        let shipment_id = format!("Shipment_{}", row_id);

        triples.push(format!(":{}  rdf:type  sc:Shipment .", shipment_id));

        // Add supply chain-specific mappings here
        // This is a template - actual fields depend on the dataset

        triples
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!("Usage: {} <input.csv> <output.ttl> [--schema order|insurance|supplychain] [--limit N]", args[0]);
        eprintln!();
        eprintln!("Examples:");
        eprintln!("  {} data.csv output.ttl --schema order --limit 10000", args[0]);
        eprintln!("  {} claims.csv insurance.ttl --schema insurance", args[0]);
        std::process::exit(1);
    }

    let input_path = &args[1];
    let output_path = &args[2];

    // Parse options
    let mut schema_type = SchemaType::Order;
    let mut limit: Option<usize> = None;

    let mut i = 3;
    while i < args.len() {
        match args[i].as_str() {
            "--schema" => {
                if i + 1 < args.len() {
                    schema_type = match args[i + 1].as_str() {
                        "order" => SchemaType::Order,
                        "insurance" => SchemaType::Insurance,
                        "supplychain" => SchemaType::SupplyChain,
                        _ => {
                            eprintln!("Invalid schema type. Use: order, insurance, or supplychain");
                            std::process::exit(1);
                        }
                    };
                    i += 2;
                } else {
                    eprintln!("--schema requires an argument");
                    std::process::exit(1);
                }
            }
            "--limit" => {
                if i + 1 < args.len() {
                    limit = Some(args[i + 1].parse().expect("Invalid limit value"));
                    i += 2;
                } else {
                    eprintln!("--limit requires a number");
                    std::process::exit(1);
                }
            }
            _ => {
                eprintln!("Unknown option: {}", args[i]);
                std::process::exit(1);
            }
        }
    }

    println!("🚀 CSV to TTL Converter");
    println!("   Input:  {}", input_path);
    println!("   Output: {}", output_path);
    println!("   Schema: {:?}", schema_type);
    if let Some(n) = limit {
        println!("   Limit:  {} rows", n);
    }
    println!();

    let converter = TTLConverter::new(schema_type, limit);

    match converter.convert_csv_to_ttl(input_path, output_path) {
        Ok(_) => {
            println!("\n🎉 Success! TTL file ready for mobile graph database.");
        }
        Err(e) => {
            eprintln!("\n❌ Error: {}", e);
            std::process::exit(1);
        }
    }
}
