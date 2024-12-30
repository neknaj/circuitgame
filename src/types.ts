export interface MType {
    input_count: number;
    output_count: number;
}

export interface Gate {
    outputs: string[];
    module_name: string;
    inputs: string[];
}

export interface Module {
    name: string;
    inputs: string[];
    outputs: string[];
    gates: Gate[];
}

export interface Using {
    type_sig: MType;
}

export interface Test {
    name: string;
    type_sig: MType;
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
    module_type_list: ModuleType[];
    module_dependency: NodeDepends[];
    module_dependency_sorted: string[];
}