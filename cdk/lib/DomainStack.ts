import { Stack, StackProps } from 'aws-cdk-lib';
import { Certificate, CertificateValidation } from 'aws-cdk-lib/aws-certificatemanager';
import {
  CnameRecord,
  HostedZone,
  IHostedZone,
  MxRecord,
  RecordSet,
  RecordTarget,
  RecordType,
  TxtRecord,
} from 'aws-cdk-lib/aws-route53';
import { Construct } from 'constructs';
import * as path from 'path';
import * as fs from 'fs';

export interface DomainStackProps extends StackProps {
  readonly domainName: string;
}

// dns-records.json should follow this type
type DnsRecordType = {
  type: RecordType;
  name: string;
  values: string[];
};

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

    /* Configuring the HostedZone which was auto-created for us when; 
    we registered the domains. It's best to NOT delete the HostedZone
    which was auto-created, since the nameservers are a bit fragile
    A lot of stuff below is for email stuff and is not necessary
    for most domains */
    this.hostedZone = HostedZone.fromLookup(this, 'HostedZone', {
      domainName: props.domainName
    });
    /* Read dns-records.json, assumed to be in the path
    ../configuration/dns-records.json and each record should follow
    the type defined for DnsRecordType above */
    const dnsRecordsPath = path.join(__dirname, '../configuration/dns-records.json');
    const dnsRecordsData = JSON.parse(fs.readFileSync(dnsRecordsPath, 'utf8'));
    if (!dnsRecordsData.records || !Array.isArray(dnsRecordsData.records)) {
      throw new Error('Invalid DNS records structure: missing or invalid "records" array');
    }
    const dnsRecords: DnsRecordType[] = dnsRecordsData.records || [];

    // Validate JSON
    dnsRecords.forEach((record: DnsRecordType) => {
      if (!record.type) {
        throw new Error(`Invalid DNS record missing "type": ${JSON.stringify(record)}`);
      }
      if (!record.name) {
        throw new Error(`Invalid DNS record missing "name": ${JSON.stringify(record)}`);
      }
      if (!(record.type in RecordType)) {
        throw new Error(`Invalid "type" for DNS record: ${JSON.stringify(record)}`);
      }
    });

    // Create records from JSON
    dnsRecords.forEach((record: DnsRecordType, index: number) => {
      new RecordSet(this, `Record${index}`, {
        zone: this.hostedZone,
        recordType: record.type,
        recordName: record.name === props.domainName ? undefined : record.name,
        target: RecordTarget.fromValues(...record.values),
      });
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
