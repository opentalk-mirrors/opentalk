--
-- Indexes for user search suggestion/autocompletion
--
CREATE EXTENSION fuzzystrmatch;
CREATE INDEX firstname_lastname_txt_like ON users (
    lower(firstname || ' ' || lastname) text_pattern_ops
);
CREATE INDEX firstname_lastname_txt_soundex ON users (soundex(lower(firstname || ' ' || lastname)));
