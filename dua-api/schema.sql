-- Create duas table for Dua API
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE duas (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    title VARCHAR(200) NOT NULL,
    arabic_text TEXT NOT NULL,
    transliteration TEXT,
    translation TEXT NOT NULL,
    reference VARCHAR(500),
    category VARCHAR(100) NOT NULL,
    tags TEXT[] DEFAULT '{}',
    audio_url TEXT,
    is_verified BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Create indexes for better performance
CREATE INDEX idx_duas_category ON duas(category);
CREATE INDEX idx_duas_verified ON duas(is_verified);
CREATE INDEX idx_duas_created_at ON duas(created_at);
CREATE INDEX idx_duas_tags ON duas USING GIN(tags);

-- Full text search index
CREATE INDEX idx_duas_search ON duas USING GIN(
    to_tsvector('english', title || ' ' || COALESCE(transliteration, '') || ' ' || translation)
);

-- Function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Trigger to automatically update updated_at
CREATE TRIGGER update_duas_updated_at 
    BEFORE UPDATE ON duas 
    FOR EACH ROW 
    EXECUTE PROCEDURE update_updated_at_column();

-- Insert some sample data
INSERT INTO duas (title, arabic_text, transliteration, translation, reference, category, tags) VALUES
('Morning Dua', 'أَصْبَحْنَا وَأَصْبَحَ الْمُلْكُ لِلَّهِ', 'Asbahna wa asbahal-mulku lillah', 'We have reached the morning and at this very time unto Allah belongs all sovereignty', 'Muslim', 'Daily Duas', ARRAY['morning', 'daily']),
('Before Eating', 'بِسْمِ اللَّهِ', 'Bismillah', 'In the name of Allah', 'Bukhari', 'Food & Drink', ARRAY['eating', 'food', 'basic']),
('After Eating', 'الْحَمْدُ لِلَّهِ الَّذِي أَطْعَمَنَا وَسَقَانَا وَجَعَلَنَا مُسْلِمِينَ', 'Alhamdulillahil-lathee atamana wa saqana wa jaalna muslimeen', 'Praise be to Allah Who has fed us and given us drink and made us Muslims', 'Abu Dawood', 'Food & Drink', ARRAY['eating', 'food', 'gratitude']),
('Before Sleep', 'بِاسْمِكَ رَبِّي وَضَعْتُ جَنْبِي وَبِكَ أَرْفَعُهُ', 'Bismika rabbee wadatu janbee wa bika arfauhu', 'In Your name my Lord, I lie down on my side, and by You I raise it up', 'Bukhari', 'Sleep', ARRAY['sleep', 'night', 'bedtime']),
('For Protection', 'أَعُوذُ بِكَلِمَاتِ اللَّهِ التَّامَّاتِ مِنْ شَرِّ مَا خَلَقَ', 'Audhu bi kalimatillahit-tammati min sharri ma khalaq', 'I seek refuge in the perfect words of Allah from the evil of what He has created', 'Muslim', 'Protection', ARRAY['protection', 'evil', 'safety']);
