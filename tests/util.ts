import { ChildProcess, spawn } from "child_process";
import keccak256 from "keccak256";
import {BigNumber, BigNumberish, ethers} from "ethers";
import BN from "bn.js";
import EC from 'elliptic';

const ec = new EC.ec('secp256k1');
const substrateContractNodePath = "./substrate-contracts-node";
export async function startContractNode() {
  const startArgs: string[] = [];
  startArgs.push("--dev", "--tmp", "-lerror,runtime::contracts=debug");
  let ls = spawn(substrateContractNodePath, startArgs);

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

  return ls;
}

export function killContractNode(childProcess: any) {
  childProcess.kill("SIGINT");
}

export function toHexString(byteArray) {
  return Array.from(byteArray, function (byte) {
    // @ts-ignore
    return ("0" + (byte & 0xff).toString(16)).slice(-2);
  }).join("");
}

export function parseHexString(str) {
  let result = [];
  while (str.length >= 8) {
    // @ts-ignore
    result.push(parseInt(str.substring(0, 8), 16));

    str = str.substring(8, str.length);
  }

  return result;
}

export function hexStringToByteArray(hexString) {
  if (hexString.length % 2 !== 0) {
    throw "Must have an even number of hex digits to convert to bytes";
  }
  var numBytes = hexString.length / 2;
  var byteArray = new Uint8Array(numBytes);
  for (var i = 0; i < numBytes; i++) {
    byteArray[i] = parseInt(hexString.substr(i * 2, 2), 16);
  }
  return byteArray;
}

export const genResourceId = (address: string): Buffer => {
  const leftPadBuf: Buffer = Buffer.alloc(6);
  const keccak = keccak256(address.toString());
  const hashedAddrBuf: Buffer = Buffer.from(keccak.buffer.slice(12));

  const chainIdType: number = getChainIdType();
  const chainIdType_buf: Buffer = Buffer.allocUnsafe(6);
  chainIdType_buf.writeUintBE(chainIdType, 0, 6);

  const resource_id: Buffer = Buffer.concat([
    leftPadBuf,
    hashedAddrBuf,
    chainIdType_buf,
  ]);

  return resource_id;
};

export const getChainIdType = (chainID: number = 1): number => {
  const CHAIN_TYPE = "0x0600";
  const chainIdType = CHAIN_TYPE + toFixedHex(chainID, 4).substr(2);
  return Number(BigInt(chainIdType));
};

/** BigNumber to hex string of specified length */
export function toFixedHex(number: BigNumberish, length: number = 32): string {
  let result =
      "0x" +
      (number instanceof Buffer
              ? number.toString("hex")
              : BigNumber.from(number.toString()).toHexString().replace("0x", "")
      ).padStart(length * 2, "0");
  if (result.indexOf("-") > -1) {
    result = "-" + result.replace("-", "");
  }
  return result;
}

export function toEncodedBinary(obj: any): string {
  return Buffer.from(JSON.stringify(obj)).toString("base64");
}


export const signMessage = (privKey: string, data: any) => {
  const key = ec.keyFromPrivate(privKey.slice(2), 'hex');
  const hash = ethers.utils.keccak256(data);
  const hashedData = ethers.utils.arrayify(hash);
  let signature = key.sign(hashedData)!;
  let expandedSig = {
    r: '0x' + signature.r.toString('hex'),
    s: '0x' + signature.s.toString('hex'),
    v: signature.recoveryParam! + 27,
  }
  let sig;
  // Transaction malleability fix if s is too large (Bitcoin allows it, Ethereum rejects it)
  try {
    sig = ethers.utils.joinSignature(expandedSig)
  } catch (e) {
    expandedSig.s = '0x' + (new BN(ec.curve.n).sub(signature.s)).toString('hex');
    expandedSig.v = (expandedSig.v === 27) ? 28 : 27;
    sig = ethers.utils.joinSignature(expandedSig)
  }

  return sig;
};
