const https = require('https');

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

}

class Test {

}

module.exports = {Compilers, Test};