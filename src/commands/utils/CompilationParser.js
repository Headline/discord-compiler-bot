import { Client } from 'discord.js'
import CompilerCommandMessage from './CompilerCommandMessage'

import url from 'url';
import fetch from 'node-fetch';

export default class CompilationParser {

    /**
     * Creates an instance of the compilation parser
     * 
     * @param {CompilerCommandMessage} message
     */
    constructor(message) {
        /**
         * @type {CompilerCommandMessage}
         */
        this.message = message;

        /**
         * @type {Client}
         */
        this.client = message.message.client;
    }

    /**
     * Figure out the args passed, it should go in the following order:
     * compile <language> < http://online.file/url | pipe data here
     * Both online file input and pipe data are optional. If the redirect input
     * character happens after pipe character just assume it's not file input
     */
    parseArguments() {
        let args = this.message.getArgs();
        args.shift();
        let argsData = {
            options: "",
            fileInput: "",
            stdin: "",
        };

        while (args.length > 0) {
            let current = args[0];

            // we encountered codeblocks, no further parsing necessary
            if (current.includes('```')) {
                break;
            }
            // accept code input via redirection operator
            else if (current == '<') {
                argsData.fileInput = args[1];
                args.shift();
            }
            // pipe operator should be last, everything after it should be stdin
            else if (current == '|') {
                do {
                    args.shift(); // kill previous
                    current = args[0];
                    argsData.stdin += current + ' ';
                } while (args.length > 1 && !args[1].includes('```'));
                args = []; // stop parsing
            }
            // anything else should just be an option
            else {
                argsData.options += current + ' ';
            }

            args.shift();
        }

        // cleanup whitespace
        argsData.options = argsData.options.trim();
        argsData.fileInput = argsData.fileInput.trim();
        argsData.stdin = argsData.stdin.trim();

        return argsData;
    }

    /**
     * Removes the language specifier from the input string
     * 
     * @param {string} code 
     * @return {string} cleaned code string
     */
    static cleanLanguageSpecifier(code) {
        const discordLanguages = [ "1c", "abnf", "accesslog", "actionscript", "ada", "apache", "applescript",
        "arduino", "armasm", "asciidoc", "aspectj", "autohotkey", "autoit", "avrasm",
        "awk", "axapta", "bash", "basic", "bnf", "brainfuck", "bf", "c", "cal", "capnproto", "ceylon",
        "clean", "clojure-repl", "clojure", "cmake", "coffeescript", "coq", "cos",
        "cpp", "crmsh", "crystal", "cs", "csharp", "csp", "css", "d", "dart", "delphi", "diff",
        "django", "dns", "dockerfile", "dos", "dsconfig", "dts", "dust", "ebnf",
        "elixir", "elm", "erb", "erlang-repl", "erlang", "excel", "fix", "flix", "fortran",
        "fsharp", "gams", "gauss", "gcode", "gherkin", "glsl", "go", "golo", "gradle", "groovy",
        "haml", "handlebars", "haskell", "haxe", "hsp", "htmlbars", "http", "hy", "inform7",
        "ini", "irpf90", "java", "javascript", "jboss-cli", "json", "js", "julia-repl", "julia",
        "kotlin", "lasso", "ldif", "leaf", "less", "lisp", "livecodeserver", "livescript",
        "llvm", "lsl", "lua", "makefile", "markdown", "mathematica", "matlab", "maxima",
        "mel", "mercury", "mipsasm", "mizar", "mojolicious", "monkey", "moonscript", "n1ql",
        "nginx", "nimrod", "nix", "nsis", "objectivec", "ocaml", "openscad", "oxygene",
        "parser3", "perl", "pf", "php", "pony", "powershell", "processing", "profile",
        "prolog", "protobuf", "puppet", "purebasic", "python", "py", "q", "qml", "r", "rib",
        "roboconf", "routeros", "rsl", "ruby", "ruleslanguage", "rust", "scala", "scheme",
        "scilab", "scss", "shell", "smali", "smalltalk", "sml", "sqf", "sql", "stan", "stata",
        "step21", "stylus", "subunit", "swift", "taggerscript", "tap", "tcl", "tex", "thrift",
        "tp", "twig", "typescript", "vala", "vbnet", "vbscript-html", "vbscript", "verilog",
        "vhdl", "vim", "x86asm", "xl", "xml", "xquery", "yaml", "zephir"
        ];
    
        let stop = 0;
        while (code.charAt(stop) != '\n' && code.charAt(stop) != ' ' && stop < code.length) {
            stop++;
        }
    
        let substr = code.substr(0, stop);
        for (let i = 0; i < discordLanguages.length; i++) {
            if (substr.toLowerCase() == discordLanguages[i]) {
                code = code.replace(substr, '');
                break;
            }
        }
        return code;
    }

    /**
     * Fetches the content from the given webpage to use as input text
     * 
     * @param {string} rawUrl raw text url
     */
    static async getCodeFromURL(rawUrl) {
        try {
            let fileInput = url.parse(rawUrl);

            if (!fileInput.hostname || !fileInput.protocol) {
                throw new Error(`Error requesting online code URL - Invalid hostname or protocol\n`);
            }
    
            let response = await fetch(fileInput.href);
            let data = await response.text();
            if (!response.ok) {
                throw new Error(`Error requesting online code URL - ${response.statusText}\n`);
            }

            return data;
        }
        catch (error) {
            throw (error); // rethrow to higher level
        }
    }

    /**
     * Parses the stdin block from the input text
     * 
     * @return {string} null if no block found
     */
    getStdinBlockFromText() {
        let text = this.message.message.content;

        const regex = /```([\s\S]*?)```/g;
        let match = regex.exec(text);
        if (!match)
            return null;

        // If we match again, our stdin belongs in the first match result
        let block1 = match[1].trim();
        match = regex.exec(text);
        if (!match)
            return null;
        else
            return block1;
    }

    /**
     * Parses the code from the input text
     * 
     * @return {string} null if no block found
     */
    getCodeBlockFromText() {
        let text = this.message.message.content;

        const regex = /```([\s\S]*?)```/g;
        let match = regex.exec(text);
        if (!match)
            return null;

        // If we match again, then our code belongs in the new match
        let block1 = match[1].trim();
        match = regex.exec(text);
        if (!match)
            return block1;
        else
            return match[1].trim();
    }
}