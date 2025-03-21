use super::types::*;

/// @return defined_non-func_module_list, defined_func_module_list, module_type_list
pub fn collect_modules(ast: &File) -> (Vec<String>,Vec<String>,Vec<ModuleType>) {
    let mut modules = Vec::new();
    let mut func_modules = Vec::new();
    let mut non_func_modules = Vec::new();

    // NOR:2->1 は常に宣言される
    modules.push(
        ModuleType {
            name: String::from("nor"),
            mtype: MType { input_count: 2, output_count: 1 },
        }
    );
    func_modules.push("nor".to_string());

    // ASTの中で定義されたモジュールを集める
    for component in &ast.components {
        match component {
            Component::Include(include) => {
                println!("Include: {}", include.path);
            },
            // Component::Graphical(include) => {
            //     println!("Graphical: {:#?}", include);
            // },
            Component::Module(module) => {
                modules.push(
                    ModuleType {
                        name: String::from(module.name.clone()),
                        mtype: MType { input_count: module.inputs.len(), output_count: module.outputs.len() },
                    }
                );
                if module.func {
                    func_modules.push(module.name.clone());
                } else {
                    non_func_modules.push(module.name.clone());
                }
            },
            _ => {} // モジュールでなければ何もしない
        }
    }
    return (non_func_modules,func_modules,modules);
}

pub fn check_module_name_duplicates(modules: &Vec<ModuleType>) -> Result<(),Vec<String>> {
    let mut module_names = std::collections::HashSet::new();
    let mut errors = Vec::new();
    for module in modules {
        if !module_names.insert(&module.name) {
            errors.push(format!("Defined module name Duplicated: {}",module.name));
        }
    }
    if errors.len()==0 { Ok(()) }
    else { Err(errors) }
}

pub fn check_module_gates(ast: &File, module_types: &Vec<ModuleType>) -> Result<(),Vec<String>> {
    let mut errors: Vec<String> = Vec::new();
    // moduleの一覧を作る
    let mut modules = Vec::new();
    for component in &ast.components {
        match component {
            Component::Module(module) => {
                modules.push(module);
            },
            _ => {},
        }
    }
    // idの宣言,使用に問題がないかを確認
    for module in &modules {
        // 宣言された名前の一覧
        let mut id_names = std::collections::HashSet::new();
        for input in &module.inputs {
            if !id_names.insert(input) {
                errors.push(format!("Defined id Duplicated: Input {} in {}",input,module.name));
            }
        }
        for gates in &module.gates {
            for output in &gates.outputs {
                if !id_names.insert(output) {
                    errors.push(format!("Defined id Duplicated: Gate-Out {} in {}",output,module.name));
                }
            }
        }
        // 宣言されていない名前が使われていないかの確認
        for output in &module.outputs {
            if !id_names.contains(output) {
                errors.push(format!("Undefined id used: Output {} in {}",output,module.name));
            }
        }
        for gates in &module.gates {
            for input in &gates.inputs {
                if !id_names.contains(input) {
                    errors.push(format!("Undefined id used: Gate-In {} in {}",input,module.name));
                }
            }
        }
        // func_moduleのみの処理
        if module.func {
            // 値が宣言の前で使われていないかどうかを確認
            let mut id_names = std::collections::HashSet::new();
            for input in &module.inputs {
                if !id_names.insert(input) {
                    // 前段でチェックされているのでエラーメッセージは出さない
                }
            }
            for gates in &module.gates {
                for output in &gates.outputs {
                    if !id_names.insert(output) {
                        // 前段でチェックされているのでエラーメッセージは出さない
                    }
                }
                for input in &gates.inputs {
                    if !id_names.contains(input) {
                        errors.push(format!("In a function module, a value cannot be used before it is declared: {} in {}",input,module.name));
                    }
                }
            }
        }
    }
    // moduleの呼び出しに問題がないかを確認
    for module in &modules {
        for gate in &module.gates {
            match module_types.iter().find(|m| m.name==gate.module_name).map(|m| &m.mtype) {
                Some(mtype) => { // 使われているモジュールが定義されている場合
                    // moduleのinput,outputの型を確認
                    if gate.inputs.len()!=mtype.input_count||gate.outputs.len()!=mtype.output_count {
                        errors.push(format!("Used module with unmatched type: {} expected {}->{} but got {}->{}, in {}",gate.module_name,mtype.input_count,mtype.output_count,gate.inputs.len(),gate.outputs.len(),module.name));
                    }
                },
                None => { errors.push(format!("Undefined module used: {} in {}",gate.module_name,module.name)); break; },
            }
            // func_moduleのみの処理
            if module.func {
                // 使っているモジュールもfunc_moduleかどうか確認
                match modules.iter().find(|m| m.name==gate.module_name).map(|m| &m.func) {
                    Some(func) => {
                        if !func {
                            errors.push(format!("Function modules cannot call non-function modules: {} used in {}",gate.module_name,module.name));
                        }
                    },
                    None => {
                        // 前段でチェックされているのでエラーメッセージは出さない
                    },
                }
            }
        }
    }

    if errors.len()==0 { Ok(()) }
    else { Err(errors) }
}

pub fn module_dependency(ast: &File) -> Vec<NodeDepends> {
    let mut dependency = Vec::new();
    for component in &ast.components {
        match component {
            Component::Module(module)=>{ // Moduleのみ処理
                let mut added = std::collections::HashSet::new();
                for gate in &module.gates {
                    if added.insert(&gate.module_name) { // 重複を防ぐ
                        dependency.push(NodeDepends { node: module.name.clone(), depends: gate.module_name.clone() });
                    }
                }
            },
            _ => {} // モジュールでなければ何もしない
        }
    }
    dependency
}

pub fn sort_dependency(dependency_vec: &Vec<NodeDepends>, modules: &Vec<ModuleType>) -> ResultwithWarn<Vec<String>> {
    use std::collections::{HashMap, HashSet};
    let mut warns: Vec<String> = Vec::new();
    // 依存関係のグラフを作成
    let mut dependency_graph: HashMap<String, HashSet<String>> = HashMap::new();
    let mut in_degree: HashMap<String, usize> = HashMap::new();
    // 全モジュールを初期化
    for module in modules {
        dependency_graph.entry(module.name.clone()).or_insert(HashSet::new());
        in_degree.entry(module.name.clone()).or_insert(0);
    }
    // 依存関係の設定
    for dep in dependency_vec {
        dependency_graph
            .entry(dep.node.clone())
            .or_insert(HashSet::new())
            .insert(dep.depends.clone());
        *in_degree.entry(dep.depends.clone()).or_insert(0) += 1;
    }
    // 被依存のないノードを見つけてSに追加
    let mut s: Vec<String> = in_degree
        .iter()
        .filter(|(_, &count)| count == 0)
        .map(|(node, _)| node.clone())
        .collect();
    s.sort();
    // 複数のルートモジュールがある場合は警告
    if s.len() > 1 {
        warns.push(format!("Multiple modules are not used by other modules: {}", s.join(", ")));
    }

    let mut l = Vec::new();
    let mut remaining: HashSet<_> = in_degree.keys().cloned().collect();

    // トポロジカルソートの実行
    while let Some(n) = s.pop() {
        l.push(n.clone());
        remaining.remove(&n);
        if let Some(deps) = dependency_graph.get(&n) {
            let mut sorted_deps: Vec<_> = deps.iter().collect();
            sorted_deps.sort();
            for m in sorted_deps {
                if let Some(count) = in_degree.get_mut(m) {
                    *count -= 1;
                    if *count == 0 {
                        match s.binary_search(m) {
                            Ok(_) => unreachable!(),
                            Err(pos) => s.insert(pos, m.clone()),
                        }
                    }
                }
            }
        }
    }

    // 残っているノードからループを検出
    if !remaining.is_empty() {
        let mut cycles = Vec::new();
        // 各未処理ノードからループを探索
        for start in &remaining {
            let mut visited = HashSet::new();
            let mut path = Vec::new();

            fn find_cycle(
                current: &str,
                graph: &HashMap<String, HashSet<String>>,
                visited: &mut HashSet<String>,
                path: &mut Vec<String>,
                cycles: &mut Vec<Vec<String>>,
            ) {
                if path.contains(&current.to_string()) {
                    // ループを検出
                    let cycle_start = path.iter().position(|x| x == current).unwrap();
                    let cycle = path[cycle_start..].to_vec();
                    if !cycles.contains(&cycle) {
                        cycles.push(cycle);
                    }
                    return;
                }
                if !visited.insert(current.to_string()) {
                    return;
                }
                path.push(current.to_string());
                if let Some(deps) = graph.get(current) {
                    for dep in deps {
                        find_cycle(dep, graph, visited, path, cycles);
                    }
                }
                path.pop();
            }
            find_cycle(start, &dependency_graph, &mut visited, &mut path, &mut cycles);
        }
        // エラーメッセージを構築
        let mut error_msg = String::from("Cycle detected in the dependency graph:\n");
        for cycle in cycles {
            error_msg.push_str(&format!("  {} -> {}\n",
                cycle.join(" -> "),
                cycle[0])); // ループを閉じる
        }
        return Err((vec![error_msg], warns));
    }

    Ok((l, warns))
}