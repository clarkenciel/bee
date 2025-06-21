# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a word list generation project that processes Princeton WordNet 3.0 data to create filtered word lists. The project has two main components:

1. **Rust application** (`src/main.rs`) - Basic Rust project structure
2. **Ruby word processor** (`script/build_words.rb`) - WordNet data parser and word list generator

## WordNet Data Structure

The project includes Princeton WordNet 3.0 data in two locations:
- `WordNet-3.0/dict/` - Original WordNet distribution
- `dict/` - Working copy of WordNet dictionary files

### Key WordNet Files
- `dict/data.{noun,verb,adj,adv}` - Main synset data files
- `dict/index.{noun,verb,adj,adv}` - Word index files  
- `dict/lexnames` - Lexicographer file mappings

### WordNet Data Format
Each synset line follows: `synset_offset lex_filenum ss_type w_cnt word1 lex_id1 [word2 lex_id2...] p_cnt [pointers...] | gloss`

Key fields for word extraction:
- `lex_filenum`: Lexicographer file number (02 = hex, identifies semantic category)
- `w_cnt`: Word count in synset (hexadecimal)
- `word1`: Primary word in synset
- Pointers like `~i` (instance hyponym) and `@i` (instance hypernym) indicate proper nouns

## WordNet Proper Noun Detection

Proper nouns are identified by:
1. **Lexicographer files**: 15 (noun.location), 18 (noun.person)
2. **Instance relationships**: Lines containing `~i` or `@i` pointers
3. **Capitalization patterns**: Proper nouns typically maintain capitals

## Build Commands

```bash
# Build Rust project
cargo build

# Run Rust application  
cargo run

# Process WordNet data with Ruby
ruby script/build_words.rb
```

## Word List Generation

The Ruby script `script/build_words.rb` processes WordNet data to generate `assets/words.txt` with filtered words:
- No proper nouns (excluding lex files 15, 18 and instance relationships)
- Only alphabetical characters [a-zA-Z]
- Deduplicated by lowercase
- Longer than 3 characters
- ASCII-only or convertible to ASCII

Output: `assets/words.txt` - sorted list of filtered words