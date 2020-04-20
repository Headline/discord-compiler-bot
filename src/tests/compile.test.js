import assert from 'assert'
import "regenerator-runtime/runtime.js";

import CompileCommand from '../commands/compile'

const cmd = new CompileCommand();
describe('Compile Command', function() {
    this.timeout(5000);
    it('Parse url', () => {
        let url = "http://michaelwflaherty.com/files/conversation.txt"
        let argsData = cmd.parseArguments(['<', url]);

        assert.equal(argsData.fileInput, url);
    });
    it('Parse options', () => {
        let args = ['-O3', '-std=c++11', '-fake-flag1', '-fake-flag2'];
        let expected = args.join(' ');

        let argsData = cmd.parseArguments(args);

        assert.equal(argsData.options, expected.trim());
    });
    it('Parse all url', () => {
        let url = "http://michaelwflaherty.com/files/conversation.txt"
        let args = ['-O3', '-std=c++11', '-fake-flag1', '-fake-flag2', '<', url, '|', 'testing', '1', '2', '3'];
        let options = args.slice(0, 4).join(' ').trim();
        let argsData = cmd.parseArguments(args);

        assert.equal(argsData.options, options);
        assert.equal(argsData.fileInput, url);
        assert.equal(argsData.stdin, 'testing 1 2 3');
    });
    it('Parse all standard', () => {
        let args = ['-O3', '-std=c++11', '-fake-flag1', '-fake-flag2', '|', 'testing', '1', '2', '3'];
        let options = args.slice(0, 4).join(' ').trim();
        let argsData = cmd.parseArguments(args);

        assert.equal(argsData.options, options);
        assert.equal(argsData.stdin, 'testing 1 2 3');
    });
    it('Parse stdin block from text', () => {
        const stdin = cmd.getStdinBlockFromText('```\ntesting 1 2 3\n```\n```cpp\nint main() {}\n```');
        assert.equal(stdin, 'testing 1 2 3');
    });

    it('Parse code from text', () => {
        const code = cmd.getCodeBlockFromText('```cpp\nint main() {}\n```');
        assert.equal(code, 'cpp\nint main() {}');
    });
    it('Clean language specifier', () => {
        let code = cmd.getCodeBlockFromText('```cpp\nint main() {}\n```');
        code = cmd.cleanLanguageSpecifier(code);
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

        let embed = cmd.buildResponseEmbed(msg, json);

        assert.equal(embed.color, 0x00FF00);
        assert.equal(embed.fields.length, 2);
    });
});