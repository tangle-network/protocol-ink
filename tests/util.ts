import { ChildProcess, spawn } from "child_process";

const substrateContractNodePath = "./substrate-contracts-node";
export async function startContractNode() {
  const startArgs: string[] = [];
  startArgs.push("--dev", "--tmp", "-lerror,runtime::contracts=debug");
  const ls = spawn(substrateContractNodePath, startArgs);

  ls.stdout.on("data", (data) => {
    console.log(`stdout: ${data}`);
  });

  ls.stderr.on("data", (data) => {
    console.error(`stderr: ${data}`);
  });
}
