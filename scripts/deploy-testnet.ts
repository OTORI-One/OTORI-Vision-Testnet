import { spawn } from 'child_process';
import { readFileSync } from 'fs';
import path from 'path';

const CONFIG_PATH = path.join(__dirname, '../program/arch-config.json');

async function main() {
  try {
    // Load configuration
    const config = JSON.parse(readFileSync(CONFIG_PATH, 'utf8'));
    console.log('📦 Loading deployment configuration...');

    // Build the program
    console.log('🔨 Building program...');
    await execCommand('cargo', ['build', '--release', '--features', 'program']);

    // Deploy to testnet
    console.log('🚀 Deploying to Arch Network testnet...');
    await execCommand('arch-cli', [
      'deploy',
      '--network',
      'testnet',
      '--rpc-url',
      config.validator.endpoint,
      '--program-id',
      config.program.id
    ]);

    console.log('✅ Deployment successful!');
  } catch (error) {
    console.error('❌ Deployment failed:', error);
    process.exit(1);
  }
}

function execCommand(command: string, args: string[]): Promise<void> {
  return new Promise((resolve, reject) => {
    const process = spawn(command, args, {
      stdio: 'inherit',
      shell: true
    });

    process.on('close', (code) => {
      if (code === 0) {
        resolve();
      } else {
        reject(new Error(`Command failed with exit code ${code}`));
      }
    });

    process.on('error', (err) => {
      reject(err);
    });
  });
}

main(); 