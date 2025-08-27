-- Create tables for Zakat API

-- Saved calculations table
CREATE TABLE zakat_calculations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id VARCHAR(255) NOT NULL,
    calculation_type VARCHAR(50) NOT NULL,
    input_data JSONB NOT NULL,
    result_data JSONB NOT NULL,
    zakat_amount DECIMAL(15,2) NOT NULL,
    currency VARCHAR(3) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Nisab rates table for current market prices
CREATE TABLE nisab_rates (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    metal_type VARCHAR(10) NOT NULL, -- 'gold' or 'silver'
    price_per_gram_usd DECIMAL(10,2) NOT NULL,
    nisab_grams DECIMAL(10,2) NOT NULL,
    nisab_value_usd DECIMAL(15,2) NOT NULL,
    last_updated TIMESTAMPTZ DEFAULT NOW(),
    source VARCHAR(100) NOT NULL,
    UNIQUE(metal_type)
);

-- Currency exchange rates
CREATE TABLE currency_rates (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    currency_code VARCHAR(3) NOT NULL,
    rate_to_usd DECIMAL(15,6) NOT NULL,
    last_updated TIMESTAMPTZ DEFAULT NOW(),
    source VARCHAR(100) NOT NULL,
    UNIQUE(currency_code)
);

-- Indexes for better performance
CREATE INDEX idx_zakat_calculations_user_id ON zakat_calculations(user_id);
CREATE INDEX idx_zakat_calculations_type ON zakat_calculations(calculation_type);
CREATE INDEX idx_zakat_calculations_created_at ON zakat_calculations(created_at);
CREATE INDEX idx_nisab_rates_metal_type ON nisab_rates(metal_type);
CREATE INDEX idx_currency_rates_currency ON currency_rates(currency_code);

-- Insert initial nisab rates
INSERT INTO nisab_rates (metal_type, price_per_gram_usd, nisab_grams, nisab_value_usd, source) VALUES
('gold', 65.00, 85.0, 5525.00, 'Initial Setup'),
('silver', 0.80, 595.0, 476.00, 'Initial Setup');

-- Insert initial currency rates
INSERT INTO currency_rates (currency_code, rate_to_usd, source) VALUES
('USD', 1.000000, 'Base Currency'),
('EUR', 0.850000, 'Initial Setup'),
('GBP', 0.730000, 'Initial Setup'),
('SAR', 3.750000, 'Initial Setup'),
('AED', 3.670000, 'Initial Setup'),
('PKR', 280.000000, 'Initial Setup'),
('INR', 83.000000, 'Initial Setup'),
('BDT', 110.000000, 'Initial Setup'),
('MYR', 4.700000, 'Initial Setup'),
('IDR', 15500.000000, 'Initial Setup'),
('TRY', 27.000000, 'Initial Setup'),
('EGP', 31.000000, 'Initial Setup');

-- Function to update nisab values when prices change
CREATE OR REPLACE FUNCTION update_nisab_value()
RETURNS TRIGGER AS $$
BEGIN
    NEW.nisab_value_usd = NEW.price_per_gram_usd * NEW.nisab_grams;
    NEW.last_updated = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Trigger to automatically calculate nisab value
CREATE TRIGGER update_nisab_rates_value 
    BEFORE UPDATE ON nisab_rates 
    FOR EACH ROW 
    EXECUTE PROCEDURE update_nisab_value();

-- Function to update currency rates timestamp
CREATE OR REPLACE FUNCTION update_currency_rates_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.last_updated = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Trigger to update timestamp on currency rate changes
CREATE TRIGGER update_currency_rates_timestamp 
    BEFORE UPDATE ON currency_rates 
    FOR EACH ROW 
    EXECUTE PROCEDURE update_currency_rates_timestamp();