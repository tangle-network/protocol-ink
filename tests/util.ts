import { ChildProcess, spawn } from "child_process";

const substrateContractNodePath =
  "sudo ./substrate-contracts-node";
export async function startContractNode() {
  const startArgs: string[] = [];
  startArgs.push("--tmp -lruntime=debug -linfo");
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
