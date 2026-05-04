# Security and Bug Bounty Program

## Overview

YieldLadder takes security seriously. We welcome responsible disclosure of security vulnerabilities through our bug bounty program.

## Scope

The following components are in scope for our bug bounty program:

- All smart contracts in the `contracts/` directory
- Protocol logic and economic mechanisms
- Access control and privilege escalation issues
- Fund loss or theft scenarios
- Denial of service attacks against core functionality

## Severity Levels

### Critical (Up to $50,000)
- Direct theft of user funds
- Permanent freezing of funds
- Protocol insolvency
- Privilege escalation to admin/strategist roles

### High (Up to $10,000)
- Temporary freezing of funds
- Yield calculation manipulation
- Bypass of lock periods or early exit fees
- Governance mechanism bypass

### Medium (Up to $2,500)
- Information disclosure
- Griefing attacks
- Non-critical access control issues
- Economic attacks requiring significant capital

### Low (Up to $500)
- Gas optimization issues
- Minor logic errors with no fund impact
- Documentation inconsistencies

## Submission Guidelines

1. **Do not exploit vulnerabilities** beyond what is necessary to demonstrate the issue
2. **Do not access or modify user data** that does not belong to you
3. **Report vulnerabilities promptly** via the contact method below
4. **Provide detailed reproduction steps** and impact assessment
5. **Allow reasonable time** for fixes before public disclosure

## Contact

Report security vulnerabilities to: **security@yieldladder.dev**

Include:
- Detailed description of the vulnerability
- Steps to reproduce
- Potential impact assessment
- Suggested mitigation (if known)

## Response Timeline

- **24 hours**: Initial acknowledgment
- **72 hours**: Severity assessment and bounty eligibility
- **30 days**: Target resolution for critical/high severity issues
- **90 days**: Target resolution for medium/low severity issues

## Legal Safe Harbor

We will not pursue legal action against researchers who:
- Act in good faith
- Follow responsible disclosure practices
- Do not violate any laws or regulations
- Do not compromise user privacy or data

## Out of Scope

- Social engineering attacks
- Physical attacks
- Denial of service against infrastructure
- Issues in third-party dependencies
- Previously reported vulnerabilities
- Issues requiring privileged access to user accounts

---

*This bug bounty program is subject to change. Check this document for the latest terms and contact information.*