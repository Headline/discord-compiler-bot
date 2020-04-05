import assert from 'assert'
import "regenerator-runtime/runtime.js";

import {Compilers, CompileSetup, Compiler} from '../utils/Wandbox'

const compilers = new Compilers(null);

describe('Wandbox Utils', function () {
    this.timeout(10000);
    it('Initialize compilers', async () => {
        return compilers.initialize();
    });
    it('Check invalid compiler', async () => {
        assert.ok(!compilers.isValidCompiler('c++'));
    });
    it('Check valid compiler', async () => {
        assert.ok(compilers.isValidCompiler('clang-3.9.1'));
    });
    it('Check valid language', async () => {
        assert.ok(compilers.has('c++'));
    });
    it('Check invalid language', async () => {
        assert.ok(!compilers.has('c--'));
    });
    it('Compiler blacklist', async () => {
        let blacklist = compilers.brokencompilers;
        blacklist.forEach(compiler => {
            assert.ok(!compilers.isValidCompiler(compiler));
        });
    });
    it('Compiler deduction', async () => {
        let setup = new CompileSetup('', 'c++', '', false, '', compilers);
        assert.notEqual(setup.compiler, 'c++');
    });
    it('Compilation dispatch', async () => {
        let setup = new CompileSetup('int main() {}', 'c++', '', true, '', compilers);
        let compiler = new Compiler(setup);
        return compiler.compile();
    });
});