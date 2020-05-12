import assert from 'assert'
import "regenerator-runtime/runtime.js";

import CompileCommand from '../commands/compile'
import CompilerCommandMessage from '../commands/utils/CompilerCommandMessage';
import CompilationParser from '../commands/utils/CompilationParser';

let fakeMessage = {
    content: ''
};

const msg = new CompilerCommandMessage(fakeMessage);
const parser = new CompilationParser(msg);
describe('Compile Command', function() {
    this.timeout(5000);
    it('Parse url', () => {
        let url = 'http://michaelwflaherty.com/files/conversation.txt';
        fakeMessage.content = ';compile c++ < ' + url;
        let argsData = parser.parseArguments();
        assert.equal(argsData.fileInput, url);
    });
    it('Parse options', () => {
        let args = ['-O3', '-std=c++11', '-fake-flag1', '-fake-flag2'];
        let expected = args.join(' ');

        fakeMessage.content = ';compile c++ ' + expected;
        let argsData = parser.parseArguments();

        assert.equal(argsData.options, expected.trim());
    });
    it('Parse all url', () => {
        let url = 'http://michaelwflaherty.com/files/conversation.txt';
        let stdin = 'testing 1 2 3';
        let options = '-O3 -std=c++11 -fake-flag1 -fake-flag2';
        fakeMessage.content = `;compile c++ ${options} < ${url} | ${stdin}`
        let argsData = parser.parseArguments();

        assert.equal(argsData.options, options);
        assert.equal(argsData.fileInput, url);
        assert.equal(argsData.stdin, 'testing 1 2 3');
    });
    it('Parse all standard', () => {
        let stdin = 'testing 1 2 3';
        let options = '-O3 -std=c++11 -fake-flag1 -fake-flag2';
        fakeMessage.content = `;compile c++ ${options} | ${stdin}`
        let argsData = parser.parseArguments();

        assert.equal(argsData.options, options);
        assert.equal(argsData.stdin, 'testing 1 2 3');
    });
    it('Parse stdin block from text', () => {
        fakeMessage.content = '```\ntesting 1 2 3\n```\n```cpp\nint main() {}\n```'
        const stdin = parser.getStdinBlockFromText();
        assert.equal(stdin, 'testing 1 2 3');
    });

    it('Parse code from text', () => {
        fakeMessage.content = '```\ntesting 1 2 3\n```\n```cpp\nint main() {}\n```'
        const code = parser.getCodeBlockFromText();
        assert.equal(code, 'cpp\nint main() {}');
    });
    it('Clean language specifier', () => {
        fakeMessage.content = '```cpp\nint main() {}\n```';
        let code = parser.getCodeBlockFromText();
        code = CompilationParser.cleanLanguageSpecifier(code);
        assert.equal(code, '\nint main() {}');
    });
    it('Compilation Embed', async () => {
        let json = JSON.parse('{ \"permlink\": \"98AbZMTsa5f9MwDd\", \"status\": \"0\", \"url\": \"https://someurl.com\"}')
        
        let msg = {
            message : {
                author: {
                    tag: 'awd'
                }
            }
        }

        let embed = CompileCommand.buildResponseEmbed(msg, json);

        assert.equal(embed.color, 0x00FF00);
        assert.equal(embed.fields.length, 2);
    });
});