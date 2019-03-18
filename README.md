#  Salad

Salad is a random passphrase generator written in Rust for *nix systems.

It is Free Software and released under the [GNU Affero General Public License V3](http://www.gnu.org/licenses/agpl.html).

Salad's goal is to provide a robust passphrase generator to users who want strong security. It is intended to provide flexible passphrase generation without requiring multiple specialized word lists by dynamically choosing words that match any desired criteria. It will also provide an estimate of the amount of entropy in a given passphrase, controlled by the word list and the passphrase settings.


## Installation

Salad is still in a very early state. As such there is no installer or distro specific packages. Compile the code with rustc and place the binary in your PATH, or simply call it from wherever you like. 


## Dependencies

Just the Rust standard library and a newline delimited text file to use as a word list. Salad comes with a large word list, although the words were chosen to maximize entropy rather than be particularly easy to memorize or spell. Using a custom word list with Salad is easy.


## Usage

Currently Salad will accept the name of a word list file on the command line. Otherwise it will look in **$HOME/.salad/words** first and **/etc/salad/words** second. 


## Configuration

Soon Salad will offer user selectable settings such as number of words in the passphrase, and minimum and maximum acceptable word length. Future plans are for more robust filtering based on punctuation, numbers, and perhaps more complex features. Currently settings are defaulted to 6 words with a minimum of 5 and maximum of 10 charatcers per word. When settings are available, they will be able to be set via commandline, environment variables and both system and user prefs files.


## Limitations

Currently there are no user selectable settings other than choosing the word list file via command line argument.
