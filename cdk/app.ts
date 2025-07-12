import { DomainStack } from './lib/DomainStack';
import { WebsiteStack } from './lib/WebsiteStack';
import { App } from 'aws-cdk-lib';
import * as dotenv from 'dotenv';
import * as path from 'path';

const app = new App();
dotenv.config({ path: path.join(__dirname, '../.env') });
const domainName = process.env.DOMAIN_NAME;
if (!domainName) {
  throw new Error('DOMAIN_NAME environment variable is required');
}

const domainStack = new DomainStack(app, 'DomainStack', {
  env: {
    account: process.env.CDK_DEFAULT_ACCOUNT,
    region: 'us-east-1', // Limitation of CloudFront, need us-east-1
  },
  domainName: domainName, 
});

const websiteStack = new WebsiteStack(app, 'WebsiteStack', {
  env: {
    account: process.env.CDK_DEFAULT_ACCOUNT,
    region: process.env.CDK_DEFAULT_REGION,
  },
  hostedZone: domainStack.hostedZone,
  certificate: domainStack.certificate,
});

websiteStack.addDependency(domainStack);