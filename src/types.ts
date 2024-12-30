export interface MType {
    inputCount: number;
    outputCount: number;
}

export interface Gate {
    outputs: string[];
    moduleName: string;
    inputs: string[];
}

export interface Module {
    name: string;
    inputs: string[];
    outputs: string[];
    gates: Gate[];
}

export interface Using {
    typeSig: MType;
}

export interface Test {
    name: string;
    typeSig: MType;
    patterns: {
        inputs: boolean[];
        outputs: boolean[];
    }[];
}

export interface Component {
    type: "Using" | "Module" | "Test";
    Using?: Using;
    Module?: Module;
    Test?: Test;
}

export interface File {
    components: Component[];
}

export interface ModuleType {
    name: string;
    mtype: MType;
}

export interface NodeDepends {
    node: string;
    depends: string;
}

export interface IntermediateProducts {
    warns: string[];
    errors: string[];
    ast: File;
    moduleTypeList: ModuleType[];
    moduleDependency: NodeDepends[];
    moduleDependencySorted: string[];
}