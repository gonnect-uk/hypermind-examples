# Dataset Download Instructions

## Step 1: Download DataCo Supply Chain Dataset (180K Records)

### Source: Mendeley Data
**URL**: https://data.mendeley.com/datasets/8gx2fvg2k6/5

### Steps:
1. Visit the URL above
2. Click "Download all files" (requires free Mendeley account)
3. Extract the ZIP file
4. Look for file: `DataCoSupplyChainDataset.csv` (~53 MB)
5. Move to: `datasets/raw/dataco_supply_chain.csv`

### Columns Expected (180,519 rows):
- Order Id, Order Date, Customer Id, Customer Fname, Customer Lname
- Customer City, Customer Country, Customer Segment
- Product Name, Category Name, Product Price
- Order Status, Sales, Order Item Quantity
- Shipping Mode, Days for shipping (real), Late_delivery_risk

---

## Step 2: Download Insurance Dataset

### Option A: Kaggle - Health Insurance Cross-Sell Prediction
**URL**: https://www.kaggle.com/datasets/anmolkumar/health-insurance-cross-sell-prediction

### Option B: Kaggle - US Health Insurance Dataset
**URL**: https://www.kaggle.com/datasets/teertha/ushealthinsurancedataset

### Steps:
1. Login to Kaggle (free account)
2. Download CSV file
3. Move to: `datasets/raw/insurance_claims.csv`

---

## Step 3: Download Brunel Logistics Dataset

### Source: Brunel University Figshare
**URL**: https://brunel.figshare.com/articles/dataset/Supply_Chain_Logistics_Problem_Dataset/7558679

### Files to Download:
- FreightRates.csv
- PlantPorts.csv
- ProductsPerPlant.csv
- WhCapacities.csv

### Move to:
```
datasets/raw/brunel_freight_rates.csv
datasets/raw/brunel_plant_ports.csv
datasets/raw/brunel_products_per_plant.csv
datasets/raw/brunel_warehouse_capacities.csv
```

---

## Quick Start: Use Sample Data (For Testing)

If you want to test the converter immediately, I can create sample CSV files based on the expected schema. These would have ~100 rows each for testing purposes.

**Command**:
```bash
# After downloading CSVs, convert them:
./tools/csv_to_ttl datasets/raw/dataco_supply_chain.csv \
    datasets/ttl/retail_products.ttl \
    --schema order \
    --limit 10000

# Check file size and triple count
ls -lh datasets/ttl/retail_products.ttl
grep -c "^:" datasets/ttl/retail_products.ttl
```

---

## Alternative: I Can Create Sample Datasets

If you'd like me to create sample CSV files with realistic data (100-500 rows each) for immediate testing, let me know. This would allow us to:
1. Test the converter
2. Verify TTL format is correct
3. Start building the iOS apps
4. Replace with real datasets later

**Which would you prefer?**
- A) I'll download the real datasets manually (provide the CSVs)
- B) Claude creates sample datasets now for testing (100-500 rows with realistic values)
