class CryptogramSolver
  @pos_char_to_words_map = {} of Int32 => Hash(Char, Set(String))
  @word_length_to_words_map = {} of Int32 => Set(String)

  def initialize(file_path)
    @words = File.read_lines(file_path)
    build_indices
  end
  
  def build_indices
    @words.each do |word|
      @word_length_to_words_map[word.size] = Set(String).new unless @word_length_to_words_map.has_key?(word.size)
      @word_length_to_words_map[word.size] << word

      word.each_char.each_with_index do |char, index|
        @pos_char_to_words_map[index] = {} of Char => Set(String) unless @pos_char_to_words_map.has_key?(index)
        @pos_char_to_words_map[index][char] = Set(String).new unless @pos_char_to_words_map[index].has_key? char
        @pos_char_to_words_map[index][char] << word
      end
    end
    # puts @pos_char_to_words_map.inspect
    puts "done building indices"
  end

  def find_words_by_letter_and_position(letter, letter_index_position)
    @pos_char_to_words_map[letter_index_position][letter]? || Set(String).new
  end

  def find_words_by_length(length)
    @word_length_to_words_map[length]? || Set(String).new
  end

  def solve(phrase)
    phrase = phrase.downcase
    encrypted_words = phrase.split(" ")

    letter_mappings = guess({} of Char => Char, encrypted_words)
    # puts(letter_mappings)

    letter_mappings.map do |letter_mapping|
      phrase.each_char.map {|encrypted_char| letter_mapping[encrypted_char]? || ' ' }.join
    end
  end

  def guess(letter_mapping, encrypted_words) : Array(Hash(Char,Char))
    encrypted_words = encrypted_words.clone
    encrypted_word = encrypted_words.shift?
    if encrypted_word
      words = find_candidate_word_matches(encrypted_word, letter_mapping)
      # puts "#{words.size} candidate words for #{encrypted_word}"
      word_to_letter_mappings = words.reduce({} of String => (Hash(Char, Char))) do |memo, word|
        mapping = is_word_possible_match?(word, encrypted_word, letter_mapping)
        memo[word] = mapping if mapping
        memo
      end
      word_to_letter_mappings.values.flat_map do |letter_mapping|
        guess(letter_mapping, encrypted_words).as(Array(Hash(Char,Char)))
      end
    else
      [letter_mapping]
    end
  end

  def find_candidate_word_matches(encrypted_word, letter_mapping : Hash(Char, Char)) : Set(String)
    candidate_word_set = find_words_by_length(encrypted_word.size)

    encrypted_word.each_char.each_with_index do |encrypted_char, index|
      plaintext_char = letter_mapping[encrypted_char]?
      if plaintext_char
        candidate_word_set &= find_words_by_letter_and_position(plaintext_char, index)
      end
    end

    candidate_word_set
  end

  # if word is a possible match for the encrypted_word, given the existing letter_mapping, then this function returns the
  # combined letter mappings of the word-specific letter mappings and the existing letter mappings; nil otherwise
  def is_word_possible_match?(word, encrypted_word, letter_mapping)
    # return nil unless word.size == encrypted_word.size      // not needed because find_candidate_word_matches will ensure this is true

    word_specific_letter_mapping = {} of Char => Char
    encrypted_word.each_char.each_with_index do |encrypted_char, index|
      plaintext_char = word[index]

      mapped_char = word_specific_letter_mapping[encrypted_char]?
      return nil if mapped_char && mapped_char != word[index]
      # return nil if word_specific_letter_mapping.has_key?(encrypted_char) && word_specific_letter_mapping[encrypted_char] != word[index]
      
      return nil if letter_mapping.has_key?(encrypted_char) && letter_mapping[encrypted_char] != word[index]

      word_specific_letter_mapping[encrypted_char] = word[index]
    end

    # ensure none of the mappings from letter_mapping conflict with the mappings in word_specific_letter_mapping
    # return nil if word_specific_letter_mapping.any? {|k,v| letter_mapping.has_key?(k) && letter_mapping[k] != v }   // this check is baked into the logic above

    letter_mapping.merge(word_specific_letter_mapping)
  end
end

def gen_cryptogram(input)
  words = input.split(" ")
  map = ('a'..'z').to_a.zip(('a'..'z').to_a.shuffle).to_h
  words.map {|word| word.each_char.map {|c| map[c]}.join }.join(" ")
end  

def main
  file_path = ARGV.first

  t1 = Time.now
  solver = CryptogramSolver.new(file_path)
  
  # phrase = "NIJBVO OBJO YAVWJB ABVB"    # "insert test phrase here"
  phrase = gen_cryptogram("insert test phrase here")
  # phrase = gen_cryptogram("most food is yummy")
  puts phrase

  solutions = solver.solve(phrase)
  t2 = Time.now
  puts t2-t1
  puts phrase
  puts "-" * phrase.size
  puts solutions.join("\n")
end

main