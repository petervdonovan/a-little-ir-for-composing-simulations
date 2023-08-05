CYAN='\033[0;32m'
NO_COLOR='\033[0m'

cd lf-types && cargo test && cd .. \
  && cd irlf-ser && cargo test && cd .. \
  && cd irlf-db && cargo test && cd .. \
  && cd get-rtor-impl && cargo test && cd .. \
  && printf "${CYAN}**************** ALL TESTS PASSED ****************${NO_COLOR}\n"
