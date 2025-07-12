import { WebsiteStack } from './lib/WebsiteStack';
import { App } from 'aws-cdk-lib';

const app = new App();

const websiteStack = new WebsiteStack(app, 'WebsiteStack', {
  env: {
        account: process.env.CDK_DEFAULT_ACCOUNT,
    region: process.env.CDK_DEFAULT_REGION,
  },
});
