import assert from 'assert'
import "regenerator-runtime/runtime.js";


import CompileCommand from '../commands/compile'

const cmd = new CompileCommand();
describe('Compile Command', () => {
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

});