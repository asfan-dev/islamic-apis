-- Clean up existing schema
DROP TABLE IF EXISTS search_index CASCADE;
DROP TABLE IF EXISTS dua_relations CASCADE;
DROP TABLE IF EXISTS dua_bundle_items CASCADE;
DROP TABLE IF EXISTS dua_bundles CASCADE;
DROP TABLE IF EXISTS dua_media CASCADE;
DROP TABLE IF EXISTS dua_context CASCADE;
DROP TABLE IF EXISTS dua_sources CASCADE;
DROP TABLE IF EXISTS dua_variants CASCADE;
DROP TABLE IF EXISTS dua_tag_map CASCADE;
DROP TABLE IF EXISTS dua_tags CASCADE;
DROP TABLE IF EXISTS dua_category_map CASCADE;
DROP TABLE IF EXISTS dua_categories CASCADE;
DROP TABLE IF EXISTS dua_translations CASCADE;
DROP TABLE IF EXISTS duas CASCADE;

-- Drop existing types
DROP TYPE IF EXISTS source_type_enum CASCADE;
DROP TYPE IF EXISTS authenticity_enum CASCADE;
DROP TYPE IF EXISTS invocation_time_enum CASCADE;
DROP TYPE IF EXISTS event_trigger_enum CASCADE;
DROP TYPE IF EXISTS posture_enum CASCADE;
DROP TYPE IF EXISTS hands_raising_enum CASCADE;
DROP TYPE IF EXISTS audible_mode_enum CASCADE;
DROP TYPE IF EXISTS addressing_mode_enum CASCADE;
DROP TYPE IF EXISTS media_type_enum CASCADE;
DROP TYPE IF EXISTS reciter_style_enum CASCADE;
DROP TYPE IF EXISTS license_enum CASCADE;
DROP TYPE IF EXISTS review_status_enum CASCADE;
DROP TYPE IF EXISTS relation_type_enum CASCADE;
DROP TYPE IF EXISTS status_enum CASCADE;

-- Create enum types
CREATE TYPE source_type_enum AS ENUM ('Quran', 'Hadith', 'Other');
CREATE TYPE authenticity_enum AS ENUM ('Quranic', 'Sahih', 'Hasan', 'Daif', 'Unclassified');
CREATE TYPE invocation_time_enum AS ENUM (
    'morning', 'evening', 'after_salah', 'before_sleep', 'after_waking',
    'before_wudu', 'after_wudu', 'entering_home', 'leaving_home',
    'entering_masjid', 'leaving_masjid', 'anytime'
);
CREATE TYPE event_trigger_enum AS ENUM (
    'waking_up', 'dressing', 'eating_start', 'eating_end', 'travel_start',
    'rain', 'thunder', 'grief', 'anxiety', 'illness', 'istikharah',
    'funeral', 'visiting_sick', 'protection', 'repentance'
);
CREATE TYPE posture_enum AS ENUM (
    'standing', 'sitting', 'sujud', 'after_salam', 'qunut_witr',
    'khutbah', 'tawaf', 'sai', 'any'
);
CREATE TYPE hands_raising_enum AS ENUM ('raise', 'dont_raise', 'context_dependent');
CREATE TYPE audible_mode_enum AS ENUM ('silent', 'soft', 'aloud');
CREATE TYPE addressing_mode_enum AS ENUM ('singular_first_person', 'plural_first_person', 'third_person');
CREATE TYPE media_type_enum AS ENUM ('audio', 'video', 'image', 'svg');
CREATE TYPE reciter_style_enum AS ENUM ('mujawwad', 'murattal', 'spoken');
CREATE TYPE license_enum AS ENUM ('CC0', 'CC-BY', 'Public Domain', 'All Rights Reserved');
CREATE TYPE review_status_enum AS ENUM ('unreviewed', 'reviewed', 'verified');
CREATE TYPE relation_type_enum AS ENUM ('related', 'seealso', 'replaces', 'contradicts');
CREATE TYPE status_enum AS ENUM ('active', 'draft', 'deprecated');

-- Core duas table
CREATE TABLE duas (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(500) NOT NULL,
    arabic_text TEXT NOT NULL,
    transliteration TEXT,
    translation TEXT NOT NULL,
    slug VARCHAR(500) UNIQUE NOT NULL,
    status VARCHAR(20) DEFAULT 'active',
    version INTEGER DEFAULT 1,
    popularity_score DECIMAL(3,2) DEFAULT 0.50,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Translations table
CREATE TABLE dua_translations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    dua_id UUID REFERENCES duas(id) ON DELETE CASCADE,
    language_code VARCHAR(10) NOT NULL,
    title VARCHAR(500),
    translation TEXT,
    transliteration TEXT,
    slug VARCHAR(500),
    seo_title VARCHAR(200),
    meta_description TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(dua_id, language_code)
);

-- Categories table
CREATE TABLE dua_categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(200) NOT NULL UNIQUE,
    slug VARCHAR(200) NOT NULL UNIQUE,
    description TEXT,
    parent_id UUID REFERENCES dua_categories(id),
    sort_order INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Category mapping
CREATE TABLE dua_category_map (
    dua_id UUID REFERENCES duas(id) ON DELETE CASCADE,
    category_id UUID REFERENCES dua_categories(id) ON DELETE CASCADE,
    PRIMARY KEY (dua_id, category_id)
);

-- Tags table
CREATE TABLE dua_tags (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL UNIQUE,
    slug VARCHAR(100) NOT NULL UNIQUE,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Tag mapping
CREATE TABLE dua_tag_map (
    dua_id UUID REFERENCES duas(id) ON DELETE CASCADE,
    tag_id UUID REFERENCES dua_tags(id) ON DELETE CASCADE,
    PRIMARY KEY (dua_id, tag_id)
);

-- Variants table
CREATE TABLE dua_variants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    dua_id UUID REFERENCES duas(id) ON DELETE CASCADE,
    variant_type VARCHAR(50) NOT NULL,
    arabic_text TEXT,
    transliteration TEXT,
    translation TEXT,
    notes TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Sources table
CREATE TABLE dua_sources (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    dua_id UUID REFERENCES duas(id) ON DELETE CASCADE,
    source_type source_type_enum NOT NULL,
    reference_text VARCHAR(500),
    book_name VARCHAR(200),
    chapter VARCHAR(200),
    hadith_number VARCHAR(50),
    authenticity authenticity_enum DEFAULT 'Unclassified',
    takhrij TEXT,
    isnad TEXT,
    notes TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Context table
CREATE TABLE dua_context (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    dua_id UUID REFERENCES duas(id) ON DELETE CASCADE UNIQUE,
    invocation_time invocation_time_enum[],
    event_trigger event_trigger_enum[],
    posture posture_enum[],
    repetition_count INTEGER,
    hands_raising_rule hands_raising_enum,
    audible_mode audible_mode_enum,
    addressing_mode addressing_mode_enum,
    etiquette_notes TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Media table
CREATE TABLE dua_media (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    dua_id UUID REFERENCES duas(id) ON DELETE CASCADE,
    media_type media_type_enum NOT NULL,
    url TEXT NOT NULL,
    file_path TEXT,
    file_size INTEGER,
    duration INTEGER,
    reciter_name VARCHAR(200),
    reciter_style reciter_style_enum,
    language_code VARCHAR(10),
    license license_enum DEFAULT 'All Rights Reserved',
    metadata JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Bundles table
CREATE TABLE dua_bundles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(200) NOT NULL,
    slug VARCHAR(200) NOT NULL UNIQUE,
    description TEXT,
    bundle_type VARCHAR(50),
    is_ruqyah BOOLEAN DEFAULT FALSE,
    metadata JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Bundle items (fixed: removed duplicate id field)
CREATE TABLE dua_bundle_items (
    bundle_id UUID REFERENCES dua_bundles(id) ON DELETE CASCADE,
    dua_id UUID REFERENCES duas(id) ON DELETE CASCADE,
    sort_order INTEGER DEFAULT 0,
    repetitions INTEGER DEFAULT 1,
    notes TEXT,
    PRIMARY KEY (bundle_id, dua_id)
);

-- Relations table
CREATE TABLE dua_relations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source_dua_id UUID REFERENCES duas(id) ON DELETE CASCADE,
    target_dua_id UUID REFERENCES duas(id) ON DELETE CASCADE,
    relation_type relation_type_enum NOT NULL,
    notes TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(source_dua_id, target_dua_id, relation_type)
);

-- Search index table (simplified - without vector column for now)
CREATE TABLE search_index (
    dua_id UUID REFERENCES duas(id) ON DELETE CASCADE PRIMARY KEY,
    text_vector tsvector,
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Create indexes
CREATE INDEX idx_duas_slug ON duas(slug);
CREATE INDEX idx_duas_status ON duas(status);
CREATE INDEX idx_duas_popularity ON duas(popularity_score DESC);
CREATE INDEX idx_dua_translations_lang ON dua_translations(language_code);
CREATE INDEX idx_dua_context_invocation ON dua_context USING GIN(invocation_time);
CREATE INDEX idx_dua_context_event ON dua_context USING GIN(event_trigger);
CREATE INDEX idx_search_text ON search_index USING GIN(text_vector);

-- Insert sample data for testing
INSERT INTO dua_categories (name, slug, description, sort_order) VALUES
('Daily Life', 'daily-life', 'Duas for everyday activities', 1),
('Salah & Adhkar', 'salah-adhkar', 'Prayer-related supplications', 2),
('Morning & Evening', 'morning-evening', 'Morning and evening remembrances', 3),
('Protection', 'protection', 'Duas for protection and safety', 4),
('Forgiveness', 'forgiveness', 'Seeking forgiveness', 5);

INSERT INTO dua_tags (name, slug) VALUES
('Essential', 'essential'),
('Sunnah', 'sunnah'),
('Quranic', 'quranic'),
('Daily', 'daily'),
('Protection', 'protection');

INSERT INTO dua_bundles (name, slug, description, bundle_type) VALUES
('Morning Adhkar', 'morning-adhkar', 'Complete morning remembrance routine', 'adhkar'),
('Evening Adhkar', 'evening-adhkar', 'Complete evening remembrance routine', 'adhkar'),
('After Salah', 'after-salah', 'Duas to recite after prayer', 'adhkar');

-- Insert a sample dua
INSERT INTO duas (title, arabic_text, transliteration, translation, slug, popularity_score) VALUES
('Bismillah', 'بِسْمِ اللَّهِ الرَّحْمَٰنِ الرَّحِيمِ', 'Bismillahir Rahmanir Raheem', 'In the name of Allah, the Most Gracious, the Most Merciful', 'bismillah', 1.00),
('Morning Awakening Dua', 'الْحَمْدُ لِلَّهِ الَّذِي أَحْيَانَا بَعْدَ مَا أَمَاتَنَا وَإِلَيْهِ النُّشُورُ', 'Alhamdu lillahil-ladhi ahyana ba''da ma amatana wa ilayhin-nushur', 'All praise is for Allah who gave us life after having taken it from us and unto Him is the resurrection', 'morning-awakening', 0.90);

-- Add category mappings for sample duas
INSERT INTO dua_category_map (dua_id, category_id) 
SELECT d.id, c.id FROM duas d, dua_categories c 
WHERE d.slug = 'bismillah' AND c.slug = 'daily-life';

INSERT INTO dua_category_map (dua_id, category_id) 
SELECT d.id, c.id FROM duas d, dua_categories c 
WHERE d.slug = 'morning-awakening' AND c.slug = 'morning-evening';