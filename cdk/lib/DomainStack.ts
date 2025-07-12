import { Stack, StackProps } from 'aws-cdk-lib';
import { Certificate, CertificateValidation } from 'aws-cdk-lib/aws-certificatemanager';
import {
  HostedZone,
  IHostedZone,
} from 'aws-cdk-lib/aws-route53';
import { Construct } from 'constructs';

export interface DomainStackProps extends StackProps {
  readonly domainName: string;
}


/* This defines all the stuff within the purview of a "domain"; in
concrete terms that is the Route53 HostedZone and a Certificate */
export class DomainStack extends Stack {
  certificate: Certificate;
  hostedZone: IHostedZone;

  constructor(scope: Construct, id: string, props: DomainStackProps) {
    super(scope, id, {
      ...props,
      crossRegionReferences: true,
    });
    /* Currently CDK cringily requires certificates to be made in 
    us-east-1, even if the CloudFront distribution is in another
    region. 
    Ref: https://github.com/aws/aws-cdk/issues/25343 */
    if (props.env?.region !== 'us-east-1') {
      throw new Error('CertificateStack must be in us-east-1');
    }

    /* Fetch the HostedZone which was auto-created for us when
    we registered the domain. It's best to NOT to manage the 
    creation/deletion of the whole HostedZone through CDK, unless
    the domain is also in CDK: the nameservers which are auto-
    configured for us are a bit fragile. */
    this.hostedZone = HostedZone.fromLookup(this, 'HostedZone', {
      domainName: props.domainName
    });

    /* Certificate for HTTPS; this gets hooked up to the CloudFront 
    distribution in the WebsiteStack */
    this.certificate = new Certificate(this, 'SiteCertificate', {
      domainName: props.domainName,
      certificateName: props.domainName,
      subjectAlternativeNames: [`*.${props.domainName}`],
      validation: CertificateValidation.fromDns(this.hostedZone),
    });
  }
}
