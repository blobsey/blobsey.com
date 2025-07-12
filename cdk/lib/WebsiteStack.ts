import { BlockPublicAccess, Bucket } from 'aws-cdk-lib/aws-s3';
import { Construct } from 'constructs';
import { RemovalPolicy, Stack, StackProps } from 'aws-cdk-lib';
import { Distribution, PriceClass } from 'aws-cdk-lib/aws-cloudfront';
import { S3BucketOrigin } from 'aws-cdk-lib/aws-cloudfront-origins';
import { BucketDeployment, Source } from 'aws-cdk-lib/aws-s3-deployment';
import { ARecord, IHostedZone, RecordTarget } from 'aws-cdk-lib/aws-route53';
import { Certificate } from 'aws-cdk-lib/aws-certificatemanager';
import { CloudFrontTarget } from 'aws-cdk-lib/aws-route53-targets';

export interface WebsiteStackProps extends StackProps {
  readonly hostedZone: IHostedZone;
  readonly certificate: Certificate;
}
export class WebsiteStack extends Stack {
  constructor(scope: Construct, id: string, props: WebsiteStackProps) {
    super(scope, id, {
      ...props,
      crossRegionReferences: true,
    });

    const domainName = props.hostedZone.zoneName;

    const bucket = new Bucket(this, 'Bucket', {
      removalPolicy: RemovalPolicy.DESTROY,
      autoDeleteObjects: true,
      blockPublicAccess: BlockPublicAccess.BLOCK_ALL,
    });

    /* CloudFront Distribution; this basically turns the S3 bucket into a
    navigable website with a real URL and HTTPS */
    const distribution = new Distribution(this, 'CloudFrontDistribution', {
      defaultBehavior: {
        origin: S3BucketOrigin.withOriginAccessControl(bucket),
      },
      certificate: props.certificate,
      domainNames: [domainName, `www.${domainName}`],
      priceClass: PriceClass.PRICE_CLASS_100, // Only US and Europe (Racism)
      comment: 'Personal website',
      defaultRootObject: 'index.html',
      errorResponses: [
        {
          httpStatus: 404,
          responseHttpStatus: 404,
          responsePagePath: '/error.html',
        },
        {
          httpStatus: 403,
          responseHttpStatus: 404,
          responsePagePath: '/error.html',
        },
      ],
    });

    new ARecord(this, 'ApexRecord', {
      zone: props.hostedZone,
      target: RecordTarget.fromAlias(new CloudFrontTarget(distribution))
    });

    new ARecord(this, 'WWWRecord', {
      zone: props.hostedZone,
      recordName: 'www',
      target: RecordTarget.fromAlias(new CloudFrontTarget(distribution))
    });

    // Deploy static files to the S3 bucket
    new BucketDeployment(this, 'DeployWebsite', {
      sources: [Source.asset('../website')],
      destinationBucket: bucket,
      distribution: distribution,
      distributionPaths: ['/*'],
    });
  }
}
