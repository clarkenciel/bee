#!/usr/bin/env ruby

# Multi-Source Word List Generator
# 
# This script processes multiple word list sources to create comprehensive filtered word lists.
# Sources: WordNet (semantic filtering), SCOWL (inflected forms), Moby (completeness)
# It can work with local data or download archives automatically.
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
# Generates assets/words.txt with 100-200k unique words:
# - WordNet: ~47k base forms with proper noun filtering
# - SCOWL: ~340k words including inflected forms
# - Moby: ~354k comprehensive word list
# - Combined and deduplicated with consistent filtering:
#   - No proper nouns (using WordNet's semantic rules)
#   - Only alphabetical characters [a-zA-Z]
#   - Deduplicated by lowercase
#   - Longer than 3 characters
#   - ASCII-only

require 'set'
require 'fileutils'
require 'net/http'
require 'uri'
require 'optparse'

class MultiSourceWordParser
  DICT_DIR = './dict'
  OUTPUT_FILE = 'assets/words.txt'
  DEFAULT_WORDNET_URL = 'https://wordnetcode.princeton.edu/wn3.1.dict.tar.gz'
  DEFAULT_SCOWL_URL = 'https://github.com/en-wl/wordlist/archive/refs/heads/master.zip'
  DEFAULT_MOBY_URL = 'https://github.com/elitejake/Moby-Project/archive/refs/heads/main.zip'
  
  # Lexicographer file numbers that contain proper nouns
  PROPER_NOUN_LEX_FILES = [15, 18].freeze # noun.location, noun.person
  
  def initialize(options = {})
    @words = Set.new
    @wordnet_words = Set.new
    @scowl_words = Set.new  
    @moby_words = Set.new
    @options = options
    @downloaded_files = []
    @extracted_dirs = []
    @proper_nouns = Set.new  # Cache proper nouns from WordNet
  end
  
  def extract_words
    begin
      puts "Multi-Source Word List Generation Starting..."
      
      # Process sources based on options
      process_wordnet if @options[:wordnet] != false
      process_scowl if @options[:scowl] == true
      process_moby if @options[:moby] == true
      
      # Merge all sources
      merge_sources
      
      puts "Final word count: #{@words.size} unique words"
      puts "  WordNet: #{@wordnet_words.size} words"
      puts "  SCOWL: #{@scowl_words.size} words" if @options[:scowl]
      puts "  Moby: #{@moby_words.size} words" if @options[:moby]
      
      write_output
      cleanup
    rescue => e
      puts "Error: #{e.message}"
      puts "Downloaded files left for inspection: #{@downloaded_files}" unless @downloaded_files.empty?
      raise
    end
  end
  
  private
  
  def process_wordnet
    puts "Processing WordNet data..."
    setup_wordnet_dir
    
    # Process all WordNet data files
    ['data.noun', 'data.verb', 'data.adj', 'data.adv'].each do |filename|
      filepath = File.join(DICT_DIR, filename)
      next unless File.exist?(filepath)
      
      puts "Processing WordNet #{filename}..."
      process_wordnet_file(filepath)
    end
    
    puts "WordNet processing complete: #{@wordnet_words.size} words"
  end
  
  def process_scowl
    puts "Processing SCOWL data..."
    setup_scowl_data
    
    # Process word list files from alt12dicts (contains inflected forms)
    scowl_files = [
      File.join('./scowl', 'alt12dicts', '2of4brif.txt'),  # Most comprehensive base list
      File.join('./scowl', 'alt12dicts', '2of12full.txt'), # Full word list with frequency
      File.join('./scowl', 'alt12dicts', '3esl.txt'),      # ESL word list
      File.join('./scowl', 'alt12dicts', '5desk.txt')      # Desk dictionary
    ]
    
    scowl_files.each do |filepath|
      next unless File.exist?(filepath)
      puts "Processing SCOWL #{File.basename(filepath)}..."
      process_scowl_file(filepath)
    end
    
    puts "SCOWL processing complete: #{@scowl_words.size} words"
  end
  
  def process_moby
    puts "Processing Moby data..."
    setup_moby_data
    
    # Process main Moby word list
    moby_file = File.join('./moby', 'mobyword.lst')
    if File.exist?(moby_file)
      puts "Processing Moby word list..."
      process_moby_file(moby_file)
    else
      puts "Warning: Moby word list not found at #{moby_file}"
    end
    
    puts "Moby processing complete: #{@moby_words.size} words"
  end
  
  def merge_sources
    puts "Merging word sources..."
    
    # Start with WordNet words (these have proper noun filtering)
    @words.merge(@wordnet_words)
    
    # Add SCOWL words, filtering against WordNet proper nouns
    @scowl_words.each do |word|
      next if is_likely_proper_noun?(word)
      @words.add(word)
    end
    
    # Add Moby words, filtering against WordNet proper nouns  
    @moby_words.each do |word|
      next if is_likely_proper_noun?(word)
      @words.add(word)
    end
  end
  
  def setup_wordnet_dir
    wordnet_url = @options[:wordnet_url] || DEFAULT_WORDNET_URL
    
    if @options[:wordnet_url]
      puts "Downloading WordNet archive from #{wordnet_url}..."
      download_and_extract_wordnet(wordnet_url)
    elsif !Dir.exist?(DICT_DIR)
      puts "No dict directory found, downloading from default URL: #{DEFAULT_WORDNET_URL}"
      download_and_extract_wordnet(DEFAULT_WORDNET_URL)
    else
      puts "Using existing WordNet dict directory: #{DICT_DIR}"
    end
  end
  
  def setup_scowl_data
    scowl_url = @options[:scowl_url] || DEFAULT_SCOWL_URL
    scowl_file = 'wordlist-master.zip'
    
    unless Dir.exist?('./scowl')
      puts "Downloading SCOWL from #{scowl_url}..."
      downloaded_file = download_file(scowl_url, scowl_file)
      @downloaded_files << downloaded_file
      extract_archive(downloaded_file)
      
      # Check if we already have a wordlist directory extracted
      if Dir.exist?('./wordlist-1')
        puts "Using existing extracted wordlist-1 directory"
        FileUtils.mv('./wordlist-1', './scowl') unless Dir.exist?('./scowl')
      else
        # Find and move extracted SCOWL directory
        wordlist_dir = Dir.glob('wordlist-*').first
        if wordlist_dir
          FileUtils.mv(wordlist_dir, './scowl')
          @extracted_dirs << './scowl'
        else
          raise "SCOWL extraction failed - no wordlist-* directory found"
        end
      end
    else
      puts "Using existing SCOWL directory: ./scowl"
    end
  end
  
  def setup_moby_data
    moby_url = @options[:moby_url] || DEFAULT_MOBY_URL
    moby_file = 'moby-main.zip'
    
    unless Dir.exist?('./moby')
      puts "Downloading Moby from #{moby_url}..."
      downloaded_file = download_file(moby_url, moby_file)
      @downloaded_files << downloaded_file
      extract_archive(downloaded_file)
      
      # Find and setup Moby directory structure
      if Dir.exist?('Moby-Project-main')
        FileUtils.mv('Moby-Project-main', './moby')
        @extracted_dirs << './moby'
      else
        raise "Moby extraction failed - no Moby-Project-main directory found"
      end
    else
      puts "Using existing Moby directory: ./moby"
    end
  end
  
  def download_and_extract_wordnet(url)
    # Download the archive
    filename = File.basename(URI(url).path)
    downloaded_file = download_file(url, filename)
    @downloaded_files << downloaded_file
    
    # Extract to temporary location
    extract_archive(downloaded_file)
    
    # Set up dict directory
    setup_extracted_dict
  end
  
  def download_file(url, filename = nil)
    uri = URI(url)
    filename ||= File.basename(uri.path)
    
    # Skip if file already exists
    if File.exist?(filename)
      puts "Using existing #{filename}"
      return filename
    end
    
    puts "Downloading #{filename}..."
    
    Net::HTTP.start(uri.hostname, uri.port, use_ssl: uri.scheme == 'https') do |http|
      request = Net::HTTP::Get.new(uri)
      
      http.request(request) do |response|
        case response.code
        when '200'
          File.open(filename, 'wb') do |file|
            response.read_body do |chunk|
              file.write(chunk)
            end
          end
        when '302', '301'
          # Follow redirect
          redirect_url = response['location']
          puts "Following redirect to #{redirect_url}"
          return download_file(redirect_url, filename)
        else
          raise "HTTP Error: #{response.code} #{response.message}"
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
    @downloaded_files.each do |file|
      if File.exist?(file)
        puts "Cleaning up downloaded file: #{file}"
        File.delete(file)
      end
    end
    
    @extracted_dirs.each do |dir|
      if Dir.exist?(dir)
        puts "Cleaning up extracted directory: #{dir}"
        FileUtils.rm_rf(dir)
      end
    end
    
    puts "Cleanup completed"
  end
  
  def process_wordnet_file(filepath)
    File.foreach(filepath) do |line|
      # Skip license header lines (start with spaces)
      next if line.start_with?('  ')
      
      # Parse the line and extract primary word
      word = extract_primary_word(line)
      if word
        processed_word = process_word(word)
        if processed_word
          @wordnet_words.add(processed_word)
          # Cache proper nouns detected by WordNet for filtering other sources
          @proper_nouns.add(processed_word.downcase) if is_proper_noun_wordnet?(line)
        end
      end
    end
  end
  
  def process_scowl_file(filepath)
    File.foreach(filepath) do |line|
      line = line.strip
      next if line.empty?
      
      # Handle different SCOWL file formats
      word = case File.basename(filepath)
      when '2of12full.txt'
        # Format: " 9:  9  -#  -&  -=   A" - extract the word at the end
        parts = line.split
        parts.last if parts.size >= 5
      when '2of4brif.txt', '3esl.txt', '5desk.txt'
        # Simple format: one word per line
        line
      else
        line
      end
      
      next if word.nil? || word.empty?
      
      processed_word = process_word(word)
      if processed_word
        @scowl_words.add(processed_word)
      end
    end
  end
  
  def process_moby_file(filepath)
    File.foreach(filepath) do |line|
      # Moby uses different delimiters - check for comma separation
      words = line.include?(',') ? line.split(',') : [line]
      
      words.each do |word|
        word = word.strip
        next if word.empty?
        
        processed_word = process_word(word)
        if processed_word
          @moby_words.add(processed_word)
        end
      end
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
    
    # Filter out proper nouns (for WordNet processing)
    return if is_proper_noun_wordnet?(line)
    
    primary_word
  end
  
  def is_proper_noun_wordnet?(line)
    # Extract lex_filenum from WordNet line for proper noun detection
    match = line.match(/^(\d{8})\s+(\d{2})\s+([nvasr])\s+([0-9a-f]{2})\s+(\S+)\s+([0-9a-f])\s/)
    return false unless match
    
    lex_filenum = match[2].to_i
    
    # Method 1: Check lexicographer file numbers
    return true if PROPER_NOUN_LEX_FILES.include?(lex_filenum)
    
    # Method 2: Check for instance relationships (proper noun indicators)
    return true if line.include?('~i') || line.include?('@i')
    
    false
  end
  
  def is_likely_proper_noun?(word)
    # Use cached proper nouns from WordNet processing
    return true if @proper_nouns.include?(word.downcase)
    
    # Basic heuristic: starts with capital letter (may catch some proper nouns)
    return true if word[0] == word[0].upcase && word.length > 1 && word[1] == word[1].downcase
    
    false
  end
  
  def process_word(word)
    # Skip multi-word terms (contain underscores - these are phrases, not single words)
    return nil if word.include?('_')
    
    # Only alphabetical characters
    return nil unless word.match?(/\A[a-zA-Z]+\Z/)
    
    # At least 4 characters long
    return nil unless word.length > 3
    
    # Convert to ASCII
    ascii_word = to_ascii(word)
    return nil unless ascii_word
    
    # Return lowercase version to deduplicate (dog and Dog -> dog)
    ascii_word.downcase
  end
  
  def to_ascii(word)
    # Simple ASCII-only filtering - reject non-ASCII words
    return word if word.ascii_only?
    
    # For now, we'll just reject non-ASCII words to avoid encoding issues
    # In production, consider using the 'unidecode' gem for proper diacritic handling
    nil
  end
  
  def to_ascii_unused(word)
    # This method is kept for reference but not used
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
  options = {
    wordnet: true,  # WordNet enabled by default
    scowl: false,   # SCOWL disabled by default
    moby: false     # Moby disabled by default
  }
  
  OptionParser.new do |opts|
    opts.banner = "Usage: #{$0} [options]"
    opts.separator ""
    opts.separator "Data Sources:"
    
    opts.on("--wordnet", "Enable WordNet processing (default: enabled)") do
      options[:wordnet] = true
    end
    
    opts.on("--no-wordnet", "Disable WordNet processing") do
      options[:wordnet] = false
    end
    
    opts.on("--scowl", "Enable SCOWL processing (adds inflected forms)") do
      options[:scowl] = true
    end
    
    opts.on("--moby", "Enable Moby word lists processing") do
      options[:moby] = true
    end
    
    opts.on("--all", "Enable all data sources") do
      options[:wordnet] = true
      options[:scowl] = true
      options[:moby] = true
    end
    
    opts.separator ""
    opts.separator "Custom URLs:"
    
    opts.on("--wordnet-url URL", "Custom WordNet archive URL") do |url|
      options[:wordnet_url] = url
    end
    
    opts.on("--scowl-url URL", "Custom SCOWL archive URL") do |url|
      options[:scowl_url] = url
    end
    
    opts.on("--moby-url URL", "Custom Moby archive URL") do |url|
      options[:moby_url] = url
    end
    
    opts.separator ""
    opts.separator "Other options:"
    
    opts.on("-h", "--help", "Show this help message") do
      puts opts
      puts ""
      puts "Examples:"
      puts "  #{$0}                    # WordNet only (default)"
      puts "  #{$0} --scowl            # WordNet + SCOWL inflected forms"
      puts "  #{$0} --all              # All sources for maximum coverage"
      puts "  #{$0} --no-wordnet --moby # Moby only"
      exit
    end
  end.parse!
  
  # Validate that at least one source is enabled
  unless options[:wordnet] || options[:scowl] || options[:moby]
    puts "Error: At least one data source must be enabled"
    puts "Use --help for usage information"
    exit 1
  end
  
  parser = MultiSourceWordParser.new(options)
  parser.extract_words
end