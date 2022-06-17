import {ChildProcess, spawn} from 'child_process';

export function startContractNode() {
    const startArgs: string[] = [];
    startArgs.push("--tmp -lruntime=debug -linfo")
    const ls = spawn( './artifacts/substrate-contracts-node-linux/substrate-contracts-node',
        []);

    ls.stdout.on('data', (data) => {
        console.log(`printing out data`);
        console.log(`stdout: ${data}`);
    });

    ls.stderr.on('data', (data) => {
        console.log(`printing out error`);
        console.error(`stderr: ${data}`);
    });
}
