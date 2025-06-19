#!/usr/bin/env ruby

# WordNet Word List Generator
# 
# This script processes Princeton WordNet data to create filtered word lists.
# It can work with local WordNet data or download archives automatically.
#
# ## WordNet Data Format Understanding
#
# ### Synset Format
# Each synset line follows: synset_offset lex_filenum ss_type w_cnt word1 lex_id1 [word2 lex_id2...] p_cnt [pointers...] | gloss
#
# Key fields for word extraction:
# - lex_filenum: Lexicographer file number (identifies semantic category)
# - w_cnt: Word count in synset (hexadecimal)
# - word1: Primary word in synset (what we extract)
# - Pointers like ~i (instance hyponym) and @i (instance hypernym) indicate proper nouns
#
# ### CRITICAL: Multi-Word Term Handling
# WordNet stores multi-word terms with underscores (e.g., "African_elephant", "American_dream")
# 
# WRONG APPROACH: Removing underscores creates concatenated pseudo-words like "africanamericanvernacularenglish"
# CORRECT APPROACH: Exclude any words containing underscores entirely
# 
# This reduces word count from ~70k to ~47k properly filtered single words.
#
# ### Proper Noun Detection (Dual Approach Required)
#
# Method 1: Lexicographer Files
# - File 15 (noun.location): Geographic locations and spatial positions  
# - File 18 (noun.person): Person names and mythical beings
#
# Method 2: Instance Relationships
# - ~i (instance hyponym): Points from general concept to specific instance
# - @i (instance hypernym): Points from specific instance to general concept
# - Lines containing either indicate proper nouns/named entities
#
# Both methods must be used together for complete proper noun filtering.
#
# ### Word Filtering Pipeline
# Apply filters in this exact order:
# 1. Skip underscored words (multi-word phrases) - MOST IMPORTANT
# 2. Apply lexicographer file filtering (files 15, 18)
# 3. Apply instance relationship filtering (~i, @i patterns)
# 4. Validate alphabetical characters only [a-zA-Z]
# 5. Enforce minimum length > 3 characters
# 6. Deduplicate by lowercase conversion
# 7. ASCII-only validation
#
# ## Usage Examples
#
# # Use existing local dict directory
# ruby build_words.rb
#
# # Download from default URL if no local dict exists
# ruby build_words.rb
#
# # Download from custom URL
# ruby build_words.rb --url https://wordnetcode.princeton.edu/wn3.1.dict.tar.gz
#
# # Show help
# ruby build_words.rb --help
#
# ## Results
# Generates assets/words.txt with ~46,932 unique words:
# - No proper nouns (excluding lex files 15, 18 and instance relationships)
# - Only alphabetical characters [a-zA-Z]
# - Deduplicated by lowercase
# - Longer than 3 characters
# - ASCII-only

require 'set'
require 'fileutils'
require 'net/http'
require 'uri'
require 'optparse'

class WordNetParser
  DICT_DIR = './dict'
  OUTPUT_FILE = 'assets/words.txt'
  DEFAULT_URL = 'https://wordnetcode.princeton.edu/wn3.1.dict.tar.gz'
  
  # Lexicographer file numbers that contain proper nouns
  PROPER_NOUN_LEX_FILES = [15, 18].freeze # noun.location, noun.person
  
  def initialize(url = nil)
    @words = Set.new
    @url = url
    @downloaded_archive = nil
    @extracted_dir = nil
  end
  
  def extract_words
    begin
      setup_dict_dir
      
      puts "Extracting words from WordNet data..."
      
      # Process all data files
      ['data.noun', 'data.verb', 'data.adj', 'data.adv'].each do |filename|
        filepath = File.join(DICT_DIR, filename)
        next unless File.exist?(filepath)
        
        puts "Processing #{filename}..."
        process_file(filepath)
      end
      
      puts "Found #{@words.size} unique words"
      write_output
      
      # Clean up only if everything succeeded
      cleanup
    rescue => e
      puts "Error: #{e.message}"
      puts "Archive left on disk for inspection: #{@downloaded_archive}" if @downloaded_archive
      raise
    end
  end
  
  private
  
  def setup_dict_dir
    if @url
      puts "Downloading WordNet archive from #{@url}..."
      download_and_extract
    elsif !Dir.exist?(DICT_DIR)
      raise "No dict directory found and no URL provided. Please provide a URL or ensure #{DICT_DIR} exists."
    else
      puts "Using existing dict directory: #{DICT_DIR}"
    end
  end
  
  def download_and_extract
    # Download the archive
    @downloaded_archive = download_file(@url)
    
    # Extract to temporary location
    extract_archive(@downloaded_archive)
    
    # Set up dict directory
    setup_extracted_dict
  end
  
  def download_file(url)
    uri = URI(url)
    filename = File.basename(uri.path)
    
    puts "Downloading #{filename}..."
    
    Net::HTTP.start(uri.hostname, uri.port, use_ssl: uri.scheme == 'https') do |http|
      request = Net::HTTP::Get.new(uri)
      
      http.request(request) do |response|
        raise "HTTP Error: #{response.code} #{response.message}" unless response.code == '200'
        
        File.open(filename, 'wb') do |file|
          response.read_body do |chunk|
            file.write(chunk)
          end
        end
      end
    end
    
    puts "Downloaded #{filename}"
    filename
  end
  
  def extract_archive(filename)
    puts "Extracting #{filename}..."
    
    case filename.downcase
    when /\.tar\.gz$/, /\.tgz$/
      system("tar -xzf #{filename}") or raise "Failed to extract #{filename}"
    when /\.tar$/
      system("tar -xf #{filename}") or raise "Failed to extract #{filename}"
    when /\.zip$/
      system("unzip #{filename}") or raise "Failed to extract #{filename}"
    else
      raise "Unsupported archive format: #{filename}"
    end
    
    puts "Extracted #{filename}"
  end
  
  def setup_extracted_dict
    # The archive should extract directly to a dict/ directory
    if Dir.exist?('dict')
      puts "Found extracted dict directory"
      @extracted_dir = 'dict'
      
      # Normalize paths for comparison
      dict_path = File.expand_path('dict')
      target_path = File.expand_path(DICT_DIR)
      
      if dict_path != target_path
        # If there's already a DICT_DIR, remove it
        if Dir.exist?(DICT_DIR)
          FileUtils.rm_rf(DICT_DIR)
        end
        
        FileUtils.mv('dict', DICT_DIR)
        puts "Moved dict to #{DICT_DIR}"
      else
        puts "Using extracted dict directory directly"
      end
    else
      raise "No dict directory found in extracted archive"
    end
  end
  
  def cleanup
    if @downloaded_archive && File.exist?(@downloaded_archive)
      puts "Cleaning up downloaded archive: #{@downloaded_archive}"
      File.delete(@downloaded_archive)
    end
    
    if @extracted_dir && Dir.exist?(@extracted_dir)
      puts "Cleaning up extracted directory: #{@extracted_dir}"
      FileUtils.rm_rf(@extracted_dir)
    end
    
    puts "Cleanup completed"
  end
  
  def process_file(filepath)
    File.foreach(filepath) do |line|
      # Skip license header lines (start with spaces)
      next if line.start_with?('  ')
      
      # Parse the line and extract primary word
      word = extract_primary_word(line)
      add_word(word) if word
    end
  end
  
  def extract_primary_word(line)
    # WordNet format: synset_offset lex_filenum ss_type w_cnt word1 lex_id1 [...]
    # Using the regex we discussed earlier
    match = line.match(/^(\d{8})\s+(\d{2})\s+([nvasr])\s+([0-9a-f]{2})\s+(\S+)\s+([0-9a-f])\s/)
    return unless match
    
    synset_offset = match[1]
    lex_filenum = match[2].to_i
    ss_type = match[3] 
    w_cnt_hex = match[4]
    primary_word = match[5]
    lex_id = match[6]
    
    # Filter out proper nouns
    return if is_proper_noun?(lex_filenum, line)
    
    primary_word
  end
  
  def is_proper_noun?(lex_filenum, line)
    # Method 1: Check lexicographer file numbers
    return true if PROPER_NOUN_LEX_FILES.include?(lex_filenum)
    
    # Method 2: Check for instance relationships (proper noun indicators)
    return true if line.include?('~i') || line.include?('@i')
    
    false
  end
  
  def add_word(word)
    # Skip multi-word terms (contain underscores - these are phrases, not single words)
    return if word.include?('_')
    
    # Only alphabetical characters
    return unless word.match?(/\A[a-zA-Z]+\Z/)
    
    # Longer than 3 characters
    return unless word.length > 3
    
    # Convert to ASCII
    ascii_word = to_ascii(word)
    return unless ascii_word
    
    # Add lowercase version to deduplicate (dog and Dog -> dog)
    @words.add(ascii_word.downcase)
  end
  
  def to_ascii(word)
    # Mapping for common non-ASCII characters to ASCII equivalents
    ascii_map = {
      # Lowercase vowels with diacritics
      '�' => 'a', '�' => 'a', '�' => 'a', '�' => 'a', '�' => 'a', '�' => 'a', '' => 'a',
      '�' => 'e', '�' => 'e', '�' => 'e', '�' => 'e', '' => 'e',
      '�' => 'i', '�' => 'i', '�' => 'i', '�' => 'i', '+' => 'i',
      '�' => 'o', '�' => 'o', '�' => 'o', '�' => 'o', '�' => 'o', '�' => 'o', 'M' => 'o',
      '�' => 'u', '�' => 'u', '�' => 'u', '�' => 'u', 'k' => 'u',
      '�' => 'y', '�' => 'y',
      
      # Uppercase vowels with diacritics  
      '�' => 'A', '�' => 'A', '�' => 'A', '�' => 'A', '�' => 'A', '�' => 'A', ' ' => 'A',
      '�' => 'E', '�' => 'E', '�' => 'E', '�' => 'E', '' => 'E',
      '�' => 'I', '�' => 'I', '�' => 'I', '�' => 'I', '*' => 'I',
      '�' => 'O', '�' => 'O', '�' => 'O', '�' => 'O', '�' => 'O', '�' => 'O', 'L' => 'O',
      '�' => 'U', '�' => 'U', '�' => 'U', '�' => 'U', 'j' => 'U',
      '�' => 'Y', 'x' => 'Y',
      
      # Other common characters
      '�' => 'n', '�' => 'N',
      '�' => 'c', '�' => 'C',
      '�' => 'ae', '�' => 'AE',
      'S' => 'oe', 'R' => 'OE',
      '�' => 'ss',
      '' => 'd', '' => 'D',
      'B' => 'l', 'A' => 'L',
      '[' => 's', 'Z' => 'S',
      'z' => 'z', 'y' => 'Z',
      '|' => 'z', '{' => 'Z'
    }
    
    converted = word.chars.map do |char|
      if char.ascii_only?
        char
      elsif ascii_map[char]
        ascii_map[char]
      else
        # Cannot convert this character to ASCII
        return nil
      end
    end.join
    
    converted
  end
  
  def write_output
    # Create assets directory if it doesn't exist
    FileUtils.mkdir_p('assets')
    
    puts "Writing #{@words.size} words to #{OUTPUT_FILE}..."
    
    File.open(OUTPUT_FILE, 'w') do |file|
      @words.sort.each { |word| file.puts(word) }
    end
    
    puts "Done! Words written to #{OUTPUT_FILE}"
  end
end

# Command line argument parsing
if __FILE__ == $0
  options = {}
  
  OptionParser.new do |opts|
    opts.banner = "Usage: #{$0} [options]"
    
    opts.on("-u", "--url URL", "WordNet archive URL (default: #{WordNetParser::DEFAULT_URL})") do |url|
      options[:url] = url
    end
    
    opts.on("-h", "--help", "Show this help message") do
      puts opts
      exit
    end
  end.parse!
  
  # Use provided URL or default if none specified and no local dict exists
  url = options[:url]
  if !url && !Dir.exist?(WordNetParser::DICT_DIR)
    puts "No local dict directory found, using default URL: #{WordNetParser::DEFAULT_URL}"
    url = WordNetParser::DEFAULT_URL
  end
  
  parser = WordNetParser.new(url)
  parser.extract_words
end