import { ChildProcess, spawn } from "child_process";

const substrateContractNodePath =
  "./substrate-contracts-node";
/*const substrateContractNodePath =
    "/Users/Damilare/Documents/Webb/new/substrate-contracts-node/target/release/substrate-contracts-node";*/
export async function startContractNode() {
  const startArgs: string[] = [];
  startArgs.push('--dev', '--tmp', '-lruntime=debug')
  //startArgs.push("--tmp -lruntime=debug -linfo");
  const ls = spawn(substrateContractNodePath, startArgs);

  ls.stdout.on("data", (data) => {
    console.log(`stdout: ${data}`);
  });

  ls.stderr.on("data", (data) => {
    console.error(`stderr: ${data}`);
  });

  const sleep = (waitTimeInMs) =>
    new Promise((resolve) => setTimeout(resolve, waitTimeInMs));
  await sleep(5000).then(() => {
    console.log("sleeping");
  });
}
