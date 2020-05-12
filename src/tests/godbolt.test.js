import "regenerator-runtime/runtime.js";
import assert from 'assert';
import { Godbolt, GodboltSetup } from '../utils/apis/Godbolt'


let godbolt = new Godbolt(null);
describe('Godbolt', function() {
    this.timeout(5000);
    it('Initialize', async () => {
        return godbolt.initialize();
    });
    it('Find language by alias', () => {
        let lang = godbolt.findLanguageByAlias('c++')
        assert.equal(lang, godbolt.get('c++'));
    });
    it('Get default compiler', () => {
        let lang = godbolt.findLanguageByAlias('c++');
        assert.ok(lang.getDefaultCompiler());
    });

    it ('Language validation by object', () => {
        let lang = godbolt.findLanguageByAlias('c++');
        assert.ok(godbolt.isValidLanguage(lang));
    });
    it ('Language validation by id', () => {
        let lang = godbolt.findLanguageByAlias('c++');
        assert.ok(godbolt.isValidLanguage(lang.id));
    });
    it ('Compiler validation by object', () => {
        let compiler = godbolt.findLanguageByAlias('c++').getDefaultCompiler();
        assert.ok(godbolt.isValidCompiler(compiler));
    });
    it ('Compiler validation by id', () => {
        let compiler = godbolt.findLanguageByAlias('c++').getDefaultCompiler();
        assert.ok(godbolt.isValidCompiler(compiler.id));
    });

    it ('Godbolt setup by object', () => {
        let compiler = godbolt.findLanguageByAlias('c++')
        new GodboltSetup(godbolt, 'int sum(int a,int b){return a+b;}', compiler, '-O3');
    });
    it ('Godbolt setup by language.id', () => {
        new GodboltSetup(godbolt, 'int sum(int a,int b){return a+b;}', 'c++', '-O3');
    });
    it ('Godbolt setup by compiler.id', () => {
        new GodboltSetup(godbolt, 'int sum(int a,int b){return a+b;}', 'g101', '-O3');
    });

    it ('Godbolt raw dispatch', async () => {
        let setup = new GodboltSetup(godbolt, 'int sum(int a,int b){return a+b;}', 'g101', '-O3');
        return setup.dispatch();
    });
    it ('Godbolt language dispatch', async () => {
        let setup = new GodboltSetup(godbolt, 'int sum(int a,int b){return a+b;}', 'c++', '-O3');
        return setup.dispatch();
    });
});