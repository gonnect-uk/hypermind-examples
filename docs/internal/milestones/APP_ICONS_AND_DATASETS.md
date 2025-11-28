# iOS App Icons & Real-Life Datasets Plan

## 📱 App Icons (SF Symbols + Custom Design)

### App 1: **Insurance Risk Analyzer**
**Icon**: Shield with checkmark + warning badge
- Primary color: #FF6B35 (Safety Orange)
- Symbol: `shield.lefthalf.filled.badge.checkmark`
- Represents: Risk assessment, compliance checking
- Easy recognition: Insurance/protection industry standard

### App 2: **Retail Product Finder**
**Icon**: Shopping cart with magnifying glass
- Primary color: #4ECDC4 (Teal Blue)
- Symbol: `cart.fill.badge.plus` + `magnifyingglass`
- Represents: Product discovery, retail shopping
- Easy recognition: E-commerce standard icon

### App 3: **Supply Chain Optimizer**
**Icon**: Truck with route/network lines
- Primary color: #95E1D3 (Mint Green)
- Symbol: `shippingbox.and.arrow.backward`
- Represents: Logistics, delivery optimization
- Easy recognition: Transportation/shipping icon

### App 4: **GraphDB Admin**
**Icon**: Graph network with gear
- Primary color: #F38181 (Coral Pink)
- Symbol: `point.3.connected.trianglepath.dotted` + `gearshape.fill`
- Represents: Technical tool, developer admin
- Easy recognition: Database/admin interface

---

## 📊 Real-Life Dataset Strategy

### Priority 1: DataCo Supply Chain (DONE - Available)
**Source**: Mendeley Data - https://data.mendeley.com/datasets/8gx2fvg2k6/5
**Records**: 180,519 real supply chain transactions
**Fields**:
- Customer demographics (age, city, country, segment)
- Order details (date, status, priority, quantity)
- Product catalog (category, subcategory, price)
- Shipping info (mode, days, warehouse)
- Financial (sales, discount, profit)

**RDF Conversion Plan**:
```turtle
# Example triple:
:Order_12345 a schema:Order ;
  schema:orderNumber "12345" ;
  schema:orderDate "2018-01-15"^^xsd:date ;
  schema:customer :Customer_789 ;
  schema:orderedItem :Product_456 ;
  schema:orderStatus "Shipped" ;
  schema:priceSpecification [
    schema:price "149.99"^^xsd:decimal ;
    schema:priceCurrency "USD"
  ] .

:Customer_789 a schema:Person ;
  schema:name "John Doe" ;
  schema:age "42"^^xsd:integer ;
  schema:addressLocality "Boston" ;
  schema:addressCountry "United States" .

:Product_456 a schema:Product ;
  schema:name "Dell Laptop XPS 15" ;
  schema:category "Electronics" ;
  schema:brand "Dell" ;
  schema:price "1299.99"^^xsd:decimal .
```

**Apps Using This**:
- Retail Product Finder (product catalog)
- Supply Chain Optimizer (logistics data)

---

### Priority 2: Insurance Claims Dataset (Kaggle)
**Source**: Kaggle - Medical/Auto Insurance Claims
**Records**: ~50,000 real insurance claims
**Fields**:
- Policy details (number, type, premium, coverage)
- Customer demographics (age, gender, location, occupation)
- Claim information (date, amount, status, reason)
- Risk factors (pre-existing conditions, driving history)

**RDF Conversion Plan**:
```turtle
:Policy_P12345 a ins:InsurancePolicy ;
  ins:policyNumber "P12345" ;
  ins:policyType "Auto" ;
  ins:premium "1200.00"^^xsd:decimal ;
  ins:coverageAmount "50000.00"^^xsd:decimal ;
  ins:policyholder :Customer_C789 ;
  ins:hasRiskFactor :RiskFactor_R1 .

:Customer_C789 a schema:Person ;
  schema:age "35"^^xsd:integer ;
  ins:drivingHistory [
    ins:accidents "0"^^xsd:integer ;
    ins:violations "1"^^xsd:integer
  ] .

:RiskFactor_R1 a ins:RiskFactor ;
  ins:riskType "Speeding Violation" ;
  ins:impactScore "0.15"^^xsd:decimal ;
  ins:description "15% premium increase" .

:Claim_CL456 a ins:InsuranceClaim ;
  ins:claimNumber "CL456" ;
  ins:claimDate "2023-06-15"^^xsd:date ;
  ins:claimAmount "5000.00"^^xsd:decimal ;
  ins:claimStatus "Approved" ;
  ins:relatedPolicy :Policy_P12345 .
```

**Apps Using This**:
- Insurance Risk Analyzer

---

### Priority 3: Brunel Supply Chain Logistics
**Source**: Brunel University Figshare - https://brunel.figshare.com/articles/dataset/Supply_Chain_Logistics_Problem_Dataset/7558679
**Records**: Multiple tables with warehouse/freight data
**Tables**:
- FreightRates (courier, lanes, weight gaps, rates)
- PlantPorts (warehouse-port connections)
- ProductsPerPlant (warehouse-product combinations)
- WhCapacities (warehouse order capacity)

**RDF Conversion Plan**:
```turtle
:Warehouse_ATL a sc:Warehouse ;
  schema:name "Atlanta Distribution Center" ;
  schema:addressLocality "Atlanta" ;
  schema:addressRegion "GA" ;
  sc:dailyCapacity "500"^^xsd:integer ;
  sc:currentLoad "387"^^xsd:integer ;
  sc:availableCapacity "113"^^xsd:integer .

:FreightRate_ATL_BOS a sc:FreightRate ;
  sc:origin :Warehouse_ATL ;
  sc:destination :Port_BOS ;
  sc:carrier "FedEx Ground" ;
  sc:ratePerPound "2.15"^^xsd:decimal ;
  sc:transitDays "3"^^xsd:integer ;
  sc:weightRange [
    sc:minWeight "10"^^xsd:integer ;
    sc:maxWeight "50"^^xsd:integer
  ] .

:ProductAvailability_P123_ATL a sc:ProductAvailability ;
  sc:product :Product_P123 ;
  sc:warehouse :Warehouse_ATL ;
  sc:quantityAvailable "245"^^xsd:integer ;
  sc:reorderPoint "50"^^xsd:integer .
```

**Apps Using This**:
- Supply Chain Optimizer

---

## 🛠️ CSV-to-TTL Converter Tool

### Tool Location: `tools/csv_to_ttl.rs`

```rust
// Standalone Rust tool (no dependencies on main crates)
// Usage: ./tools/csv_to_ttl datasets/raw/supply_chain.csv datasets/ttl/supply_chain.ttl --schema order

use std::fs::File;
use std::io::{BufRead, BufReader, Write};

struct TTLConverter {
    schema_type: SchemaType,
    namespace: String,
}

enum SchemaType {
    Order,        // schema:Order, schema:Product, schema:Customer
    Insurance,    // custom ins: namespace
    SupplyChain,  // custom sc: namespace
}

impl TTLConverter {
    fn convert_csv_to_ttl(&self, input_path: &str, output_path: &str) {
        // Parse CSV
        // Generate triples
        // Write TTL with proper prefixes
    }

    fn generate_prefixes(&self) -> String {
        r#"
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix schema: <https://schema.org/> .
@prefix ins: <http://example.org/insurance/> .
@prefix sc: <http://example.org/supplychain/> .

"#.to_string()
    }
}
```

### Conversion Commands:
```bash
# DataCo Supply Chain → Retail Products
./tools/csv_to_ttl datasets/raw/dataco_supply_chain.csv \
  datasets/ttl/retail_products.ttl \
  --schema order \
  --limit 10000  # First 10K records for mobile

# Insurance Claims → Insurance Policies
./tools/csv_to_ttl datasets/raw/insurance_claims.csv \
  datasets/ttl/insurance_policies.ttl \
  --schema insurance \
  --limit 5000

# Brunel Logistics → Supply Chain
./tools/csv_to_ttl datasets/raw/brunel_logistics.csv \
  datasets/ttl/supply_chain_logistics.ttl \
  --schema supplychain \
  --limit 8000
```

---

## 📦 Dataset Embedding in iOS Apps

### File Locations:
```
ios/datasets/
├── retail_products.ttl          (10K triples, ~2MB)
├── insurance_policies.ttl       (5K triples, ~1MB)
└── supply_chain_logistics.ttl   (8K triples, ~1.5MB)
```

### Bundle Resources in Xcode:
```swift
// Load TTL from app bundle
guard let ttlPath = Bundle.main.path(forResource: "retail_products", ofType: "ttl") else {
    fatalError("Dataset not found in app bundle")
}

// Parse and load into graph DB
let db = try GonnectNanoGraphDB(storageType: .inMemory)
try db.loadTTL(fromFile: ttlPath)

print("Loaded \(db.tripleCount()) triples - OFFLINE READY ✅")
```

---

## 🎨 Icon Implementation Plan

### Step 1: Generate App Icon Assets
```bash
# Use SF Symbols app to export custom icons
# Export at these sizes:
- 1024x1024 (App Store)
- 180x180 (iPhone)
- 120x120 (iPhone)
- 167x167 (iPad Pro)
- 152x152 (iPad)
- 76x76 (iPad)
- 60x60 (iPhone)
- 40x40 (iPhone)
- 29x29 (Settings)
- 20x20 (Notification)
```

### Step 2: Add to Assets.xcassets
```
ios/InsuranceRiskAnalyzer/Assets.xcassets/AppIcon.appiconset/
ios/RetailProductFinder/Assets.xcassets/AppIcon.appiconset/
ios/SupplyChainOptimizer/Assets.xcassets/AppIcon.appiconset/
ios/GraphDBAdmin/Assets.xcassets/AppIcon.appiconset/
```

### Step 3: Update Info.plist
```xml
<key>CFBundleDisplayName</key>
<string>Risk Analyzer</string>
<key>CFBundleIcons</key>
<dict>
    <key>CFBundlePrimaryIcon</key>
    <dict>
        <key>CFBundleIconFiles</key>
        <array>
            <string>AppIcon</string>
        </array>
    </dict>
</dict>
```

---

## ✅ Implementation Checklist

### Week 1: Data Acquisition & Conversion
- [ ] Download DataCo Supply Chain CSV (180K records)
- [ ] Download Kaggle Insurance Claims CSV (50K records)
- [ ] Download Brunel Logistics CSVs
- [ ] Build `csv_to_ttl` converter tool
- [ ] Generate `retail_products.ttl` (10K triples)
- [ ] Generate `insurance_policies.ttl` (5K triples)
- [ ] Generate `supply_chain_logistics.ttl` (8K triples)
- [ ] Validate TTL files with Apache Jena riot tool
- [ ] Delete `movies_catalog.ttl` (synthetic data)

### Week 2: Icon Design & App Renaming
- [ ] Design custom app icons (SF Symbols + Figma)
- [ ] Export icon assets at all required sizes
- [ ] Rename `ComplianceGuardian` → `InsuranceRiskAnalyzer`
- [ ] Rename `SmartSearch` → `RetailProductFinder`
- [ ] Rename `ProductConfigurator` → `SupplyChainOptimizer`
- [ ] Update all Xcode project settings
- [ ] Add icons to Assets.xcassets
- [ ] Test icon appearance on iOS simulator

### Week 3: App UI Rebuild
- [ ] Insurance Risk Analyzer with real insurance data
- [ ] Retail Product Finder with real product catalog
- [ ] Supply Chain Optimizer with real logistics data
- [ ] GraphDBAdmin mobile-optimized graph viz

### Week 4: Testing & Demo
- [ ] Load real TTL datasets (verify offline mode)
- [ ] Test all SPARQL queries with real data
- [ ] Measure query performance (should be <10 µs)
- [ ] Record demo videos showing offline capability
- [ ] Create App Store screenshots

---

## 🚀 Expected Results

### Dataset Sizes:
- **Retail Products**: 10,000 triples (~2 MB)
- **Insurance Policies**: 5,000 triples (~1 MB)
- **Supply Chain**: 8,000 triples (~1.5 MB)
- **Total**: 23,000 triples (~4.5 MB) in app bundle

### Performance:
- Cold start: <500 ms (load 23K triples)
- Query time: 2.78 µs avg
- Memory: ~6 MB (24 bytes/triple)
- 100% offline (airplane mode works)

### User Experience:
- Clear persona-driven apps
- Beautiful, recognizable icons
- Real-world data (not synthetic)
- "How It Works" shows RDF reasoning
- Faster than ChatGPT (100,000x)
- No internet required

---

**Next Step**: Build the `csv_to_ttl` converter tool and start downloading real datasets!
