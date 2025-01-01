var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
var __generator = (this && this.__generator) || function (thisArg, body) {
    var _ = { label: 0, sent: function() { if (t[0] & 1) throw t[1]; return t[1]; }, trys: [], ops: [] }, f, y, t, g = Object.create((typeof Iterator === "function" ? Iterator : Object).prototype);
    return g.next = verb(0), g["throw"] = verb(1), g["return"] = verb(2), typeof Symbol === "function" && (g[Symbol.iterator] = function() { return this; }), g;
    function verb(n) { return function (v) { return step([n, v]); }; }
    function step(op) {
        if (f) throw new TypeError("Generator is already executing.");
        while (g && (g = 0, op[0] && (_ = 0)), _) try {
            if (f = 1, y && (t = op[0] & 2 ? y["return"] : op[0] ? y["throw"] || ((t = y["return"]) && t.call(y), 0) : y.next) && !(t = t.call(y, op[1])).done) return t;
            if (y = 0, t) op = [op[0] & 2, t.value];
            switch (op[0]) {
                case 0: case 1: t = op; break;
                case 4: _.label++; return { value: op[1], done: false };
                case 5: _.label++; y = op[1]; op = [0]; continue;
                case 7: op = _.ops.pop(); _.trys.pop(); continue;
                default:
                    if (!(t = _.trys, t = t.length > 0 && t[t.length - 1]) && (op[0] === 6 || op[0] === 2)) { _ = 0; continue; }
                    if (op[0] === 3 && (!t || (op[1] > t[0] && op[1] < t[3]))) { _.label = op[1]; break; }
                    if (op[0] === 6 && _.label < t[1]) { _.label = t[1]; t = op; break; }
                    if (t && _.label < t[2]) { _.label = t[2]; _.ops.push(op); break; }
                    if (t[2]) _.ops.pop();
                    _.trys.pop(); continue;
            }
            op = body.call(thisArg, _);
        } catch (e) { op = [6, e]; y = 0; } finally { f = t = 0; }
        if (op[0] & 5) throw op[1]; return { value: op[0] ? op[1] : void 0, done: true };
    }
};
import init, { Compile, CompilerIntermediateProducts, Test } from './circuitgame_lib.js';
import { isIntermediateProducts } from './typeGuards.js';
function CompileAsTypescriptResult(code) {
    var result_from_rust = JSON.parse(CompilerIntermediateProducts(code));
    // 型チェックと変換を行う
    if (!isIntermediateProducts(result_from_rust)) {
        throw new Error('Rustからの返り値が期待する形式と一致しません');
    }
    return result_from_rust;
}
function fetchTextFile(url) {
    return __awaiter(this, void 0, void 0, function () {
        var response, text, error_1;
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0:
                    _a.trys.push([0, 3, , 4]);
                    return [4 /*yield*/, fetch(url)];
                case 1:
                    response = _a.sent();
                    // レスポンスが成功しているかを確認
                    if (!response.ok) {
                        throw new Error("HTTP Error: ".concat(response.status));
                    }
                    return [4 /*yield*/, response.text()];
                case 2:
                    text = _a.sent();
                    return [2 /*return*/, text];
                case 3:
                    error_1 = _a.sent();
                    console.error("テキストファイルの取得中にエラーが発生しました:", error_1);
                    throw error_1;
                case 4: return [2 /*return*/];
            }
        });
    });
}
function run() {
    return __awaiter(this, void 0, void 0, function () {
        var input, result, test_result, _i, _a, name_1;
        return __generator(this, function (_b) {
            switch (_b.label) {
                case 0: return [4 /*yield*/, init()];
                case 1:
                    _b.sent();
                    return [4 /*yield*/, fetchTextFile("./sample.ncg")];
                case 2:
                    input = _b.sent();
                    {
                        console.log("< Input >");
                        result = CompileAsTypescriptResult(input);
                        console.log(result);
                        console.log(result.module_dependency_sorted[0]);
                        test_result = JSON.parse(Test(input));
                        console.log(test_result);
                        for (_i = 0, _a = Object.keys(test_result.test_result); _i < _a.length; _i++) {
                            name_1 = _a[_i];
                            console.log("test: ".concat(name_1));
                            console.table(test_result.test_result[name_1]);
                        }
                        console.log(Compile(input, result.module_dependency_sorted[0]));
                    }
                    return [2 /*return*/];
            }
        });
    });
}
run();
