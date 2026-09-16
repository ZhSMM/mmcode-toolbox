import { invokeCmd } from './index';
export interface CidrReq { input: string }
export interface CidrResp {
  network: string; broadcast: string | null;
  netmask: string; wildcard: string;
  total_addresses: string; usable_addresses: string;
  first_host: string | null; last_host: string | null;
  version: number; is_private: boolean;
}
export function cidrInfo(req: CidrReq) { return invokeCmd<CidrResp>('cidr_info', { req }); }

export interface IpLookupReq { ip: string }
export interface IpLookupResp {
  ip: string; version: number;
  is_private: boolean; is_loopback: boolean; is_link_local: boolean;
  is_multicast: boolean; is_unspecified: boolean; is_documentation: boolean;
  reverse_dns_hint: string;
}
export function ipInfo(req: IpLookupReq) { return invokeCmd<IpLookupResp>('ip_info', { req }); }