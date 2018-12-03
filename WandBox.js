const https = require('https');
const fetch = require('node-fetch');
/**
 * A class designed to fetch & hold the list of valid
 * compilers from wandbox.
 */
class Compilers {
    /**
     * Creates a Compilers object and fetches the list of valid
     * compilers from wandbox. You may pass a finished callback
     * for logging.
     * 
     * @param {Function} finishedCallback 
     */
    constructor(finishedCallback) {
        https.get('https://wandbox.org/api/list.json', (response) => {
            let data = '';

            response.on('data', (chunk) => {
                data += chunk;
            })
            response.on('end', () => {
                this.compilers = JSON.parse(data);
                finishedCallback();

            })
        }).on("error", (err) => {
            console.log("Error: " + err.message);
        });
    }

    initialize() {
        this.compilerinfo = [];
        this.languages = [];

        this.compilers.forEach((obj) => {
            let lang = obj.language.toLowerCase();
            let compiler = obj.name;
            if (this.languages.indexOf(lang) < 0) {
                this.languages.push(lang);
                this.compilerinfo[lang] = [];
            }

            this.compilerinfo[lang].push(compiler);
        });
    }

    getCompilers(language) {
        if (this.languages.indexOf(language) < 0) { // no such lanuage
            return "None";
        }
        return this.compilerinfo[language];
    }

    isValidCompiler(compiler) {
        for (let i = 0; i < this.languages.length; i++) {
            for (let j = 0; j < this.compilerinfo[this.languages[i]].length; j++) {
                if (compiler == this.compilerinfo[this.languages[i]][j])
                    return true;
            }
        }
        return false;
    }

}

/**
 * Class which represents all the settings and information for a single compilation
 * request. This should be built and used in coordination with Compiler. 
 */
class CompileSetup {
    /**
     * Creates a compilation setup for usage with the Compiler object.
     * You may pass a language instead of a compiler for the second parameter,
     * and it will be compiled with the first compiler found in the list. The compiler
     * used is #1 on the menu for ;compilers <lang>.
     * @param {String} code 
     * @param {String} compiler 
     * @param {String} stdin 
     * @param {Boolean} save 
     * @param {Compilers} compilers 
     */
    constructor(code, compiler, stdin, save, compiler_option_raw, compilers) {
        this.code = code;
        this.stdin = stdin;
        this.save = save;
        this.compiler_option_raw = compiler_option_raw.split(' ').join('\n'); // joined by comma per doc spec

        let comp = compiler.toLowerCase();
        let langs = compilers.languages;
        if (langs.indexOf(comp) >= 0)
            this.compiler = compilers.compilerinfo[comp][0];
        else
            this.compiler = comp;
    }
}

/**
 * Request sender which creates and sends a CompileSetup
 */
class Compiler {
    /**
     * Creates a compilation object which compiles code.
     * 
     * @param {CompileSetup} compilesetup 
     */
    constructor(compilesetup) {
        this.compilesetup = compilesetup
    }

    /**
     * Sends a request to wandbox servers with the code, language, and compiler.
     * If the json object returned by onCompleted is null, an error has occured.
     */
    compile(onCompleted) {
        let url = "https://wandbox.org/api/compile.json";
        fetch(url, {
            method: "POST",
            body: JSON.stringify(this.compilesetup).replace('compiler_option_raw', 'compiler-option-raw'),
            headers: {
                'Content-Type': 'application/json; charset=utf-8'
            },
        })
        .then(response =>response.json())
        .then(json => onCompleted(json))
        .catch((ex) =>  {
            onCompleted(null);
        });
    }
}

module.exports = {Compilers, Compiler, CompileSetup};