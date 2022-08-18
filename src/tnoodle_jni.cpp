#include <sstream>
#include <iostream>

#include "tnoodle_jni.h"

#include "twsearch.h"
#include "puzdef.h"
#include "cmdlineops.h"
#include "parsemoves.h"
#include "util.h"
#include "readksolve.h"

template <typename T>

T withZeroVerbosity(function<T()> f) {
    int old_verbose = verbose;
    verbose = 0;

    T result = f();

    verbose = old_verbose;

    return result;
}

void withZeroVerbosity(function<void()> f) {
    withZeroVerbosity<std::string>([&]() {
        f();
        // noop return because C++ doesn't accept void as a generic return type.
        return std::string();
    });
}

string redirectStdout(function<void()> f) {
    std::stringstream buffer;
    std::streambuf* old = std::cout.rdbuf(buffer.rdbuf());

    withZeroVerbosity(f);

    std::cout.rdbuf( old );
    return buffer.str();
}

template <typename T>

T withSilentStdout(function<T()> f) {
    // block stdout from printing stuff
    std::cout.setstate(std::ios_base::failbit);

    T bootstrapResult = withZeroVerbosity(f);

    // enable stdout again, so we can fetch the actual result we care about
    std::cout.clear();

    return bootstrapResult;
}

puzdef makeSilentPuzdef(string kDefinition) {
    return withSilentStdout<puzdef>([&]() {
        return makepuzdef(kDefinition);
    });
}

std::string javaToStdString(JNIEnv* env, jstring javaString) {
    return env->GetStringUTFChars(javaString, NULL);
}

jstring stdToJavaString(JNIEnv* env, std::string stdString) {
    return env->NewStringUTF(stdString.c_str());
}

JNIEXPORT jstring JNICALL Java_org_worldcubeassociation_tnoodle_binding_TwSearchPuzzle_generateRandomState(JNIEnv* env, jobject thisObject, jstring javaKDefinition) {
    std::string kDefinition = javaToStdString(env, javaKDefinition);

    puzdef pd = makeSilentPuzdef(kDefinition);

    std::string randomPos = redirectStdout([&]() {
        showrandompos(pd);
    });

    return stdToJavaString(env, randomPos);
}

JNIEXPORT jstring JNICALL Java_org_worldcubeassociation_tnoodle_binding_TwSearchPuzzle_solveState(JNIEnv* env, jobject thisObject, jstring javaKDefinition, jstring javaKState) {
    std::string kDefinition = javaToStdString(env, javaKDefinition);
    std::string kState = javaToStdString(env, javaKState);

    puzdef pd = makeSilentPuzdef(kDefinition);

    prunetable pt = withSilentStdout<prunetable>([&]() {
        return prunetable(pd, maxmem);
    });

    ull checksum = 0;
    std::istringstream is(kState);
    std::vector<string> toks = getline(&is, checksum);

    setval ps = withSilentStdout<setval>([&]() {
        return readposition(pd, 'S', &is, checksum);
    });

    std::string noname;

    std::string solution = redirectStdout([&]() {
        solveit(pd, pt, noname, ps);
    });

    return stdToJavaString(env, solution);
}

JNIEXPORT jstring JNICALL Java_org_worldcubeassociation_tnoodle_binding_TwSearchPuzzle_invertAlgorithm(JNIEnv* env, jobject thisObject, jstring javaKDefinition, jstring javaScramble) {
    std::string kDefinition = javaToStdString(env, javaKDefinition);
    std::string scramble = javaToStdString(env, javaScramble);

    puzdef pd = makeSilentPuzdef(kDefinition);

    vector<int> moveid = withSilentStdout<vector<int>>([&]() {
        return parsemovelist(pd, scramble.c_str());
    });

    std::string inverted = redirectStdout([&]() {
        invertit(pd, moveid, scramble.c_str());
    });

    return stdToJavaString(env, inverted);
}
