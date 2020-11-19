# This script takes care of testing
set -ex

main() {
    if [ ! -z $DISABLE_TESTS ]; then
        return
    fi

    cross test --release --target $TARGET
    cross build --release --target $TARGET
}

if [ -z $TRAVIS_TAG ]; then
    main
fi
