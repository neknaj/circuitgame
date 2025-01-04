import { Compile } from './circuitgame.js';
import { elm as E, textelm as T } from './cdom.js';
import { IntermediateProducts } from './types.js';

function init(elm: HTMLDivElement,product: IntermediateProducts,module_name: string) {
    product.module_type_list
    elm.Add(E("div",{class:"input"},[]));
    elm.Add(E("div",{class:"output"},[]));
}

export default init;