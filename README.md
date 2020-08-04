#  Salad

Salad is a random passphrase generator written in Rust for *nix systems.

It is Free Software and released under the [GNU Affero General Public License V3](http://www.gnu.org/licenses/agpl.html).

Salad's goal is to provide a robust passphrase generator to users who want strong security. It is intended to provide flexible passphrase generation without requiring multiple specialized word lists by dynamically choosing words that match any desired criteria.


## Installation

Salad is still in a very early state. As such there is no installer or distro specific packages. Compile the code with rustc and place the binary in your PATH, or simply call it from wherever you like. 


## Dependencies

Just the Rust standard library and a newline delimited text file to use as a word list. Salad comes with a large word list, but using a custom word list with Salad is easy.


## Features

Salad gives you three ways of generating a random passphrase. Each method will allow you to choose the number of words in your passphrase, and the maximum and minimum length of those words.

**Method 1: Random**  
Randomly selected words (use the -r option)

**Method 2: Dynamic Mnemonic**  
A random word for use as a mnemonic will be chosen that contains a number of characters equal to the number of words you want in your passphrase. Then a random word beginning with each letter in that mnemonic will be chosen to form your passphrase (use the -m option)

**Method 3: Fixed Mnemonic**  
The same as #2 except using a mnemonic specified on the commandline (use the -M option)

Salad uses rejection sampling to select random words from a file containing a list of words. This provides a uniformly random sampling of words, without requiring a fixed size word list. The word list included with salad contains only lower case ascii letters, but files containing "words" consisting of any unicode characters should work as long as there is one word per line (blank lines are ignored and will not affect output).


## Usage

salad [OPTION]...

Generate a passphrase from a file containing a list of words.

EXAMPLE  
salad -M floyd -min 4 -max 8

DEFAULTS  
-m -n 6 -max 12 -min 5

OPTIONS  
-h, --help  
  Display usage help

-max N  
  Ignore words larger than N. N must be less than 256.

-min N  
  Ignore words smaller than N. N must be less than 256.

-n N  
  Generate a passphrase with N words. N must be less than 256.

-r  
  Generate a passphrase of random words. Mutually exclusive with -m and -M.

-m  
  Generate a passphrase using a ramdomly chosen mnemonic. Mutually exclusive with -r and -M.

-M MNEMONIC  
  Generate a passphrase using the specified mnemonic. Mutually exclusive with -r and -m. The option -n is ignored if this is used.

-w FILE  
  Use a custom word-file. If no custom word-file is provided, salad will look in **$HOME/.salad/words** first and **/etc/salad/words** second. 

