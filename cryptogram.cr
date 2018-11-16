class CryptogramSolver
  @pos_char_to_words_map = {} of Int32 => Hash(Char, Set(String))
  # @word_length_to_words_map = {} of Int32 => Set(String)
  @word_pattern_to_words = {} of Array(Int32) => Set(String)
  @number_of_branches_evaluated = 0

  def initialize(file_path)
    t1=Time.now
    @words = File.read_lines(file_path)
    build_indices
    t2=Time.now
    puts "time to build indices: #{t2-t1}"
  end
  
  def build_indices
    @words.each do |word|
      # @word_length_to_words_map[word.size] = Set(String).new unless @word_length_to_words_map.has_key?(word.size)
      # @word_length_to_words_map[word.size] << word

      pattern = word_pattern(word)
      @word_pattern_to_words[pattern] = Set(String).new unless @word_pattern_to_words.has_key?(pattern)
      @word_pattern_to_words[pattern] << word

      word.each_char.each_with_index do |char, index|
        @pos_char_to_words_map[index] = {} of Char => Set(String) unless @pos_char_to_words_map.has_key?(index)
        @pos_char_to_words_map[index][char] = Set(String).new unless @pos_char_to_words_map[index].has_key? char
        @pos_char_to_words_map[index][char] << word
      end
    end
    puts "done building indices"
  end

  def find_words_by_letter_and_position(letter, letter_index_position)
    @pos_char_to_words_map[letter_index_position][letter]? || Set(String).new
  end

  # def find_words_by_length(length)
  #   @word_length_to_words_map[length]? || Set(String).new
  # end

  def find_words_by_pattern(word_pattern)
    @word_pattern_to_words[word_pattern]? || Set(String).new
  end

  def solve(phrase)
    @number_of_branches_evaluated = 0

    phrase = phrase.downcase
    encrypted_words = phrase.split(" ")

    letter_mappings = guess({} of Char => Char, {} of Char => Char, encrypted_words)
    # puts(letter_mappings)
    puts "#{@number_of_branches_evaluated} branches evaluated"

    letter_mappings.map do |letter_mapping|
      new_phrase = phrase.each_char.map {|encrypted_char| letter_mapping[encrypted_char]? || ' ' }.join
      # puts "#{new_phrase} - #{letter_mapping.each.to_a.sort_by{|pair| pair[0] }.map{|(k,v)| "#{k}->#{v}" }.join(" ")}"
      new_phrase
    end
  end

  def guess(letter_mapping, reverse_letter_mapping, encrypted_words) : Array(Hash(Char,Char))
    encrypted_words = encrypted_words.sort_by {|word| find_candidate_word_matches(word, letter_mapping).size }
    encrypted_word = encrypted_words.shift?
    if encrypted_word
      words = find_candidate_word_matches(encrypted_word, letter_mapping)
      # puts "#{words.size} candidate words for #{encrypted_word}"
      word_to_letter_mappings = words.reduce({} of String => Tuple(Hash(Char, Char), Hash(Char, Char))) do |memo, word|
        mapping_pair = try_extend_mapping(word, encrypted_word, letter_mapping, reverse_letter_mapping)
        if mapping_pair
          memo[word] = mapping_pair
        else
          @number_of_branches_evaluated += 1    # mapping_pair was nil, so we pruned that branch
        end
        memo
      end
      word_to_letter_mappings.values.flat_map do |letter_mapping_pair|
        letter_mapping, reverse_letter_mapping = letter_mapping_pair
        guess(letter_mapping, reverse_letter_mapping, encrypted_words).as(Array(Hash(Char,Char)))
      end
    else
      @number_of_branches_evaluated += 1      # we ran out of words, so this branch is now a leaf - letter_mapping is a solution letter mapping
      [letter_mapping]
    end
  end

  def find_candidate_word_matches(encrypted_word, letter_mapping : Hash(Char, Char)) : Set(String)
    # candidate_word_set = find_words_by_length(encrypted_word.size)
    candidate_word_set = find_words_by_pattern(word_pattern(encrypted_word))

    encrypted_word.each_char.each_with_index do |encrypted_char, index|
      plaintext_char = letter_mapping[encrypted_char]?
      if plaintext_char
        candidate_word_set &= find_words_by_letter_and_position(plaintext_char, index)
      end
    end

    candidate_word_set
  end

  def word_pattern(word)
    count = 0
    char_to_number = Hash(Char,Int32).new
    word.each_char.map do |char|
      char_to_number[char]? || (char_to_number[char] = (count += 1))
    end.to_a
  end

  # if word is a possible match for the encrypted_word, given the existing letter_mapping, then this function returns the
  # combined letter mappings of the word-specific letter mappings and the existing letter mappings; nil otherwise
  # Assumes word.size == encrypted_word.size
  def try_extend_mapping(word, encrypted_word, letter_mapping, reverse_letter_mapping) : Tuple(Hash(Char, Char), Hash(Char, Char)) | Nil
    # return nil unless word.size == encrypted_word.size      // not needed because find_candidate_word_matches will ensure this is true

    letter_mapping, reverse_letter_mapping = letter_mapping.clone, reverse_letter_mapping.clone
    encrypted_word.each_char.each_with_index do |encrypted_char, index|
      plaintext_char = word[index]

      # we have a word-derived mapping: encrypted_char -> plaintext_char
      # we need to make sure it doesn't conflict with any pre-existing mappings

      # ensure none of the existing mappings conflict with the new candidate mapping
      pre_existing_mapped_char = letter_mapping[encrypted_char]?
      return nil if pre_existing_mapped_char && pre_existing_mapped_char != plaintext_char

      # ensure none of the existing reverse mappings conflict with the new candidate mapping
      pre_existing_reverse_mapped_char = reverse_letter_mapping[plaintext_char]?
      return nil if pre_existing_reverse_mapped_char && pre_existing_reverse_mapped_char != encrypted_char

      letter_mapping[encrypted_char] = plaintext_char
      reverse_letter_mapping[plaintext_char] = encrypted_char
    end

    {letter_mapping, reverse_letter_mapping}
  end
end

def gen_cryptogram(input)
  words = input.split(" ")
  map = ('a'..'z').to_a.zip(('a'..'z').to_a.shuffle).to_h
  words.map {|word| word.each_char.map {|c| map[c]}.join }.join(" ")
end  

def main
  file_path = ARGV.first

  solver = CryptogramSolver.new(file_path)
  
  # phrase = "NIJBVO OBJO YAVWJB ABVB"    # "insert test phrase here"
  phrase = gen_cryptogram("the best way to find yourself is to lose yourself in the service of others")
  # phrase = gen_cryptogram("insert test phrase here")
  # phrase = gen_cryptogram("most food is yummy")
  # phrase = "why the long face"    # ha ha! good luck!
  puts phrase

  t1 = Time.now
  solutions = solver.solve(phrase)
  t2 = Time.now
  puts t2-t1
  puts phrase
  puts "-" * phrase.size
  puts solutions.join("\n")
end

main
