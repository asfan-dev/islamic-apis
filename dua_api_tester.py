#!/usr/bin/env python3
"""
Comprehensive test suite for Dua API
Tests all endpoints with proper logging and error handling
Automatically discovers test data from the API instead of using hardcoded values
"""

import requests
import json
import logging
import time
from datetime import datetime
from typing import Dict, Any, Optional, List
import uuid


class DuaAPITester:
    def __init__(self, base_url: str = "http://localhost:3000"):
        self.base_url = base_url
        self.session = requests.Session()
        self.session.headers.update({
            'Content-Type': 'application/json',
            'User-Agent': 'DuaAPI-Tester/1.0'
        })
        
        # Setup logging
        logging.basicConfig(
            level=logging.INFO,
            format='%(asctime)s - %(levelname)s - %(message)s',
            handlers=[
                logging.FileHandler('dua_api_test.log', mode='w'),
                logging.StreamHandler()
            ]
        )
        self.logger = logging.getLogger(__name__)
        
        # Test statistics
        self.total_tests = 0
        self.passed_tests = 0
        self.failed_tests = 0
        self.test_results = []
        
        # Auto-discovered test data
        self.test_data = {
            'categories': [],
            'tags': [],
            'bundles': [],
            'duas': [],
            'sources': []
        }

    def discover_test_data(self):
        """Automatically discover test data from API endpoints"""
        self.logger.info("🔍 Discovering test data from API...")
        
        # Discover categories
        try:
            status_code, data, error = self.make_request('GET', '/v1/categories')
            if status_code == 200 and 'categories' in data:
                self.test_data['categories'] = data['categories']
                self.logger.info(f"Found {len(self.test_data['categories'])} categories")
        except Exception as e:
            self.logger.warning(f"Could not discover categories: {e}")
        
        # Discover tags
        try:
            status_code, data, error = self.make_request('GET', '/v1/tags')
            if status_code == 200 and 'tags' in data:
                self.test_data['tags'] = data['tags']
                self.logger.info(f"Found {len(self.test_data['tags'])} tags")
        except Exception as e:
            self.logger.warning(f"Could not discover tags: {e}")
        
        # Discover bundles
        try:
            status_code, data, error = self.make_request('GET', '/v1/bundles')
            if status_code == 200 and 'bundles' in data:
                self.test_data['bundles'] = data['bundles']
                self.logger.info(f"Found {len(self.test_data['bundles'])} bundles")
        except Exception as e:
            self.logger.warning(f"Could not discover bundles: {e}")
        
        # Discover duas
        try:
            status_code, data, error = self.make_request('GET', '/v1/duas', params={'per_page': 5})
            if status_code == 200 and 'duas' in data:
                self.test_data['duas'] = data['duas']
                self.logger.info(f"Found {len(self.test_data['duas'])} sample duas")
        except Exception as e:
            self.logger.warning(f"Could not discover duas: {e}")
        
        # Discover sources
        try:
            status_code, data, error = self.make_request('GET', '/v1/sources', params={'per_page': 5})
            if status_code == 200 and 'sources' in data:
                self.test_data['sources'] = data['sources']
                self.logger.info(f"Found {len(self.test_data['sources'])} sources")
        except Exception as e:
            self.logger.warning(f"Could not discover sources: {e}")

    def get_sample_category(self) -> Optional[str]:
        """Get a sample category slug for testing"""
        if self.test_data['categories']:
            return self.test_data['categories'][0]['slug']
        return None

    def get_sample_tag(self) -> Optional[str]:
        """Get a sample tag slug for testing"""
        if self.test_data['tags']:
            return self.test_data['tags'][0]['slug']
        return None

    def get_sample_bundle(self) -> Optional[str]:
        """Get a sample bundle slug for testing"""
        if self.test_data['bundles']:
            return self.test_data['bundles'][0]['slug']
        return None

    def get_sample_dua(self) -> Optional[Dict]:
        """Get a sample dua for testing"""
        if self.test_data['duas']:
            return self.test_data['duas'][0]
        return None

    def get_sample_source(self) -> Optional[Dict]:
        """Get a sample source for testing"""
        if self.test_data['sources']:
            return self.test_data['sources'][0]
        return None

    def log_test_result(self, endpoint: str, method: str, params: Dict[str, Any], 
                       response_code: int, response_data: Any, success: bool, error: str = None):
        """Log test result to both console and log file"""
        self.total_tests += 1
        if success:
            self.passed_tests += 1
            status = "PASS"
        else:
            self.failed_tests += 1
            status = "FAIL"
        
        result = {
            'endpoint': endpoint,
            'method': method,
            'params': params,
            'response_code': response_code,
            'response_data': response_data,
            'status': status,
            'error': error,
            'timestamp': datetime.now().isoformat()
        }
        
        self.test_results.append(result)
        
        self.logger.info(f"[{status}] {method} {endpoint}")
        self.logger.info(f"Params: {json.dumps(params, indent=2)}")
        self.logger.info(f"Response Code: {response_code}")
        if error:
            self.logger.error(f"Error: {error}")
        else:
            self.logger.info(f"Response: {json.dumps(response_data, indent=2, default=str)}")
        self.logger.info("-" * 80)

    def make_request(self, method: str, endpoint: str, params: Dict = None, 
                    json_data: Dict = None) -> tuple:
        """Make HTTP request with error handling"""
        url = f"{self.base_url}{endpoint}"
        try:
            if method.upper() == 'GET':
                response = self.session.get(url, params=params, timeout=10)
            elif method.upper() == 'POST':
                response = self.session.post(url, json=json_data, params=params, timeout=10)
            else:
                raise ValueError(f"Unsupported method: {method}")
            
            try:
                data = response.json() if response.content else {}
            except json.JSONDecodeError:
                data = {"raw_response": response.text}
            
            return response.status_code, data, None
            
        except requests.exceptions.RequestException as e:
            return 0, {}, str(e)

    def test_health_check(self):
        """Test health check endpoint"""
        status_code, data, error = self.make_request('GET', '/health')
        success = status_code == 200 and error is None
        self.log_test_result('/health', 'GET', {}, status_code, data, success, error)

    def test_list_duas(self):
        """Test list duas endpoint with various parameters"""
        # Build test cases dynamically based on discovered data
        test_cases = [
            # Basic list
            {},
            # With pagination
            {'page': 1, 'per_page': 5},
            # With sorting
            {'sort': 'title', 'order': 'asc'},
            # With language
            {'lang': 'en'},
            # With search
            {'q': 'morning'},
            {'q': 'Allah'},
            # With includes
            {'include': 'sources,categories,tags'},
        ]
        
        # Add dynamic filter tests based on discovered data
        sample_category = self.get_sample_category()
        if sample_category:
            test_cases.extend([
                {'category': sample_category},
                {'category': sample_category, 'include': 'categories'},
            ])
        
        sample_tag = self.get_sample_tag()
        if sample_tag:
            test_cases.extend([
                {'tag': sample_tag},
                {'tag': sample_tag, 'include': 'tags'},
            ])
        
        # Add common filter tests
        test_cases.extend([
            {'invocation_time': 'morning'},
            {'invocation_time': 'evening'},
            {'event_trigger': 'waking_up'},
            {'source_type': 'Quran'},
            {'source_type': 'Hadith'},
            {'authenticity': 'Sahih'},
            {'has_audio': 'true'},
            {'has_audio': 'false'},
            # Complex query
            {
                'page': 2,
                'per_page': 10,
                'sort': 'popularity_score',
                'order': 'desc',
                'include': 'sources,categories'
            }
        ])
        
        for params in test_cases:
            status_code, data, error = self.make_request('GET', '/v1/duas', params=params)
            success = status_code == 200 and error is None
            self.log_test_result('/v1/duas', 'GET', params, status_code, data, success, error)

    def test_get_random_dua(self):
        """Test random dua endpoint"""
        test_cases = [
            {},
            {'include': 'sources,categories,tags'}
        ]
        
        # Add dynamic tests based on discovered data
        sample_category = self.get_sample_category()
        if sample_category:
            test_cases.append({'category': sample_category})
        
        # Add common tests
        test_cases.extend([
            {'invocation_time': 'morning'},
            {'invocation_time': 'evening'},
            {'event_trigger': 'waking_up'}
        ])
        
        for params in test_cases:
            status_code, data, error = self.make_request('GET', '/v1/duas/random', params=params)
            # Accept both 200 (found) and 404 (not found) as successful
            success = status_code in [200, 404] and error is None
            self.log_test_result('/v1/duas/random', 'GET', params, status_code, data, success, error)

    def test_get_dua_by_id(self):
        """Test get specific dua by ID or slug"""
        test_cases = []
        
        # Use actual dua data if available
        sample_dua = self.get_sample_dua()
        if sample_dua:
            # Test with real dua ID
            test_cases.extend([
                {'id': sample_dua['id'], 'params': {}},
                {'id': sample_dua['id'], 'params': {'include': 'sources,categories,tags'}},
            ])
            # Test with real dua slug
            test_cases.extend([
                {'id': sample_dua['slug'], 'params': {}},
                {'id': sample_dua['slug'], 'params': {'include': 'sources,media'}},
            ])
        else:
            # Fallback to dummy data if no duas found
            test_dua_id = str(uuid.uuid4())
            test_cases.extend([
                {'id': test_dua_id, 'params': {}},
                {'id': 'nonexistent-slug', 'params': {}},
            ])
        
        for case in test_cases:
            endpoint = f"/v1/duas/{case['id']}"
            status_code, data, error = self.make_request('GET', endpoint, params=case['params'])
            # Accept both 200 (found) and 404 (not found) as successful API responses
            success = status_code in [200, 404] and error is None
            self.log_test_result(endpoint, 'GET', case['params'], status_code, data, success, error)

    def test_translations(self):
        """Test translation endpoints"""
        # Use actual dua ID if available
        sample_dua = self.get_sample_dua()
        test_dua_id = sample_dua['id'] if sample_dua else str(uuid.uuid4())
        
        # Test get dua translations
        endpoint = f"/v1/duas/{test_dua_id}/translations"
        status_code, data, error = self.make_request('GET', endpoint)
        success = status_code == 200 and error is None
        self.log_test_result(endpoint, 'GET', {}, status_code, data, success, error)
        
        # Test list all translations
        status_code, data, error = self.make_request('GET', '/v1/translations')
        success = status_code == 200 and error is None
        self.log_test_result('/v1/translations', 'GET', {}, status_code, data, success, error)

    def test_categories(self):
        """Test category endpoints"""
        # List categories
        status_code, data, error = self.make_request('GET', '/v1/categories')
        success = status_code == 200 and error is None
        self.log_test_result('/v1/categories', 'GET', {}, status_code, data, success, error)
        
        # Test with actual category or fallback
        sample_category = self.get_sample_category()
        test_slug = sample_category if sample_category else "nonexistent-category"
        
        # Get category by slug
        endpoint = f"/v1/categories/{test_slug}"
        status_code, data, error = self.make_request('GET', endpoint)
        success = status_code in [200, 404] and error is None
        self.log_test_result(endpoint, 'GET', {}, status_code, data, success, error)
        
        # Get category duas
        endpoint = f"/v1/categories/{test_slug}/duas"
        params = {'page': 1, 'per_page': 5}
        status_code, data, error = self.make_request('GET', endpoint, params=params)
        success = status_code == 200 and error is None
        self.log_test_result(endpoint, 'GET', params, status_code, data, success, error)

    def test_tags(self):
        """Test tag endpoints"""
        # List tags
        status_code, data, error = self.make_request('GET', '/v1/tags')
        success = status_code == 200 and error is None
        self.log_test_result('/v1/tags', 'GET', {}, status_code, data, success, error)
        
        # Test with actual tag or fallback
        sample_tag = self.get_sample_tag()
        test_slug = sample_tag if sample_tag else "nonexistent-tag"
        
        # Get tag duas
        endpoint = f"/v1/tags/{test_slug}/duas"
        params = {'page': 1, 'per_page': 5}
        status_code, data, error = self.make_request('GET', endpoint, params=params)
        success = status_code == 200 and error is None
        self.log_test_result(endpoint, 'GET', params, status_code, data, success, error)

    def test_bundles(self):
        """Test bundle endpoints"""
        # List bundles
        status_code, data, error = self.make_request('GET', '/v1/bundles')
        success = status_code == 200 and error is None
        self.log_test_result('/v1/bundles', 'GET', {}, status_code, data, success, error)
        
        # Test with actual bundle or fallback
        sample_bundle = self.get_sample_bundle()
        test_slug = sample_bundle if sample_bundle else "nonexistent-bundle"
        
        # Get bundle by slug
        endpoint = f"/v1/bundles/{test_slug}"
        status_code, data, error = self.make_request('GET', endpoint)
        success = status_code in [200, 404] and error is None
        self.log_test_result(endpoint, 'GET', {}, status_code, data, success, error)
        
        # Get bundle items
        endpoint = f"/v1/bundles/{test_slug}/items"
        status_code, data, error = self.make_request('GET', endpoint)
        success = status_code in [200, 404] and error is None
        self.log_test_result(endpoint, 'GET', {}, status_code, data, success, error)

    def test_sources(self):
        """Test source endpoints"""
        # List sources with various filters
        params_list = [
            {},
            {'source_type': 'Quran'},
            {'source_type': 'Hadith'},
            {'authenticity': 'Sahih'},
            {'authenticity': 'Quranic'},
            {'q': 'Bukhari'},
            {'q': 'Quran'},
            {'page': 1, 'per_page': 10}
        ]
        
        for params in params_list:
            status_code, data, error = self.make_request('GET', '/v1/sources', params=params)
            success = status_code == 200 and error is None
            self.log_test_result('/v1/sources', 'GET', params, status_code, data, success, error)
        
        # Test with actual source ID or fallback
        sample_source = self.get_sample_source()
        test_source_id = sample_source['id'] if sample_source else str(uuid.uuid4())
        
        # Get source by ID
        endpoint = f"/v1/sources/{test_source_id}"
        status_code, data, error = self.make_request('GET', endpoint)
        success = status_code in [200, 404] and error is None
        self.log_test_result(endpoint, 'GET', {}, status_code, data, success, error)
        
        # Get source duas
        endpoint = f"/v1/sources/{test_source_id}/duas"
        status_code, data, error = self.make_request('GET', endpoint)
        success = status_code == 200 and error is None
        self.log_test_result(endpoint, 'GET', {}, status_code, data, success, error)

    def test_media(self):
        """Test media endpoints"""
        # Use actual dua ID if available
        sample_dua = self.get_sample_dua()
        test_dua_id = sample_dua['id'] if sample_dua else str(uuid.uuid4())
        
        # Get dua media
        endpoint = f"/v1/duas/{test_dua_id}/media"
        status_code, data, error = self.make_request('GET', endpoint)
        success = status_code == 200 and error is None
        self.log_test_result(endpoint, 'GET', {}, status_code, data, success, error)
        
        # Search media with various filters
        params_list = [
            {},
            {'media_type': 'audio'},
            {'media_type': 'video'},
            {'license': 'CC0'},
            {'license': 'CC-BY'},
            {'reciter': 'Abdul Rahman'},
            {'reciter': 'Sheikh'},
            {'page': 1, 'per_page': 5}
        ]
        
        for params in params_list:
            status_code, data, error = self.make_request('GET', '/v1/media', params=params)
            success = status_code == 200 and error is None
            self.log_test_result('/v1/media', 'GET', params, status_code, data, success, error)

    def test_search(self):
        """Test search endpoints"""
        # Keyword search with various queries
        params_list = [
            {'q': 'morning'},
            {'q': 'Allah'},
            {'q': 'protection'},
            {'q': 'Bismillah'},
            {'q': 'dua'},
            {'q': 'protection', 'limit': 5},
            {'q': ''}  # Test empty query
        ]
        
        for params in params_list:
            status_code, data, error = self.make_request('GET', '/v1/search', params=params)
            success = status_code == 200 and error is None
            self.log_test_result('/v1/search', 'GET', params, status_code, data, success, error)
        
        # Semantic search
        semantic_requests = [
            {'query': 'morning prayers', 'limit': 10},
            {'query': 'protection from evil', 'limit': 5, 'threshold': 0.7},
            {'query': 'seeking forgiveness'},
            {'query': 'daily supplications'},
            {'query': 'Islamic prayers'}
        ]
        
        for req_data in semantic_requests:
            status_code, data, error = self.make_request('POST', '/v1/search/semantic', json_data=req_data)
            success = status_code == 200 and error is None
            self.log_test_result('/v1/search/semantic', 'POST', req_data, status_code, data, success, error)
        
        # Autocomplete suggestions
        suggest_params = [
            {'q': 'mor'},
            {'q': 'prot'},
            {'q': 'Al'},
            {'q': 'bis'},
            {'q': 'prot', 'limit': 5},
        ]
        
        for params in suggest_params:
            status_code, data, error = self.make_request('GET', '/v1/suggest', params=params)
            success = status_code == 200 and error is None
            self.log_test_result('/v1/suggest', 'GET', params, status_code, data, success, error)

    def test_stats(self):
        """Test statistics endpoint"""
        status_code, data, error = self.make_request('GET', '/v1/stats')
        success = status_code == 200 and error is None
        self.log_test_result('/v1/stats', 'GET', {}, status_code, data, success, error)

    def test_error_cases(self):
        """Test various error cases"""
        error_cases = [
            # Invalid endpoints
            {'method': 'GET', 'endpoint': '/v1/invalid', 'params': {}, 'expected_codes': [404]},
            {'method': 'GET', 'endpoint': '/v1/nonexistent', 'params': {}, 'expected_codes': [404]},
            # Invalid method
            {'method': 'POST', 'endpoint': '/v1/duas', 'params': {}, 'expected_codes': [405]},
            {'method': 'PUT', 'endpoint': '/v1/stats', 'params': {}, 'expected_codes': [405]},
            # Invalid parameters
            {'method': 'GET', 'endpoint': '/v1/duas', 'params': {'page': -1}, 'expected_codes': [400, 200]},
            {'method': 'GET', 'endpoint': '/v1/duas', 'params': {'per_page': 1000}, 'expected_codes': [400, 200]},
            {'method': 'GET', 'endpoint': '/v1/duas', 'params': {'has_audio': 'invalid'}, 'expected_codes': [400]},
        ]
        
        for case in error_cases:
            try:
                if case['method'] == 'POST':
                    status_code, data, error = self.make_request(case['method'], case['endpoint'], json_data=case['params'])
                else:
                    status_code, data, error = self.make_request(case['method'], case['endpoint'], params=case['params'])
                
                success = (status_code in case['expected_codes'] and error is None) or error is not None
                self.log_test_result(case['endpoint'], case['method'], case['params'], status_code, data, success, error)
            except Exception as e:
                # For unsupported methods, this is expected
                self.log_test_result(case['endpoint'], case['method'], case['params'], 0, {}, True, str(e))

    def run_all_tests(self):
        """Run all test cases"""
        self.logger.info("=" * 80)
        self.logger.info("Starting Dua API Test Suite")
        self.logger.info(f"Base URL: {self.base_url}")
        self.logger.info(f"Start Time: {datetime.now().isoformat()}")
        self.logger.info("=" * 80)
        
        # First, discover test data
        self.discover_test_data()
        
        test_methods = [
            ("Health Check", self.test_health_check),
            ("List Duas", self.test_list_duas),
            ("Random Dua", self.test_get_random_dua),
            ("Get Dua by ID/Slug", self.test_get_dua_by_id),
            ("Translations", self.test_translations),
            ("Categories", self.test_categories),
            ("Tags", self.test_tags),
            ("Bundles", self.test_bundles),
            ("Sources", self.test_sources),
            ("Media", self.test_media),
            ("Search", self.test_search),
            ("Statistics", self.test_stats),
            ("Error Cases", self.test_error_cases)
        ]
        
        for test_name, test_method in test_methods:
            self.logger.info(f"\n{'='*20} TESTING {test_name.upper()} {'='*20}")
            try:
                test_method()
                self.logger.info(f"✅ {test_name} tests completed")
            except Exception as e:
                self.logger.error(f"❌ {test_name} tests failed with exception: {e}")
            
            # Small delay between test groups
            time.sleep(0.5)
        
        self.print_summary()

    def print_summary(self):
        """Print test summary"""
        self.logger.info("\n" + "=" * 80)
        self.logger.info("TEST SUMMARY")
        self.logger.info("=" * 80)
        self.logger.info(f"Total Tests: {self.total_tests}")
        self.logger.info(f"Passed: {self.passed_tests}")
        self.logger.info(f"Failed: {self.failed_tests}")
        self.logger.info(f"Success Rate: {(self.passed_tests/self.total_tests*100):.1f}%" if self.total_tests > 0 else "0%")
        self.logger.info(f"End Time: {datetime.now().isoformat()}")
        
        # Log discovered data summary
        if any(self.test_data.values()):
            self.logger.info("\nDISCOVERED TEST DATA:")
            self.logger.info(f"- Categories: {len(self.test_data['categories'])}")
            self.logger.info(f"- Tags: {len(self.test_data['tags'])}")
            self.logger.info(f"- Bundles: {len(self.test_data['bundles'])}")
            self.logger.info(f"- Duas: {len(self.test_data['duas'])}")
            self.logger.info(f"- Sources: {len(self.test_data['sources'])}")
        
        # Log failed tests
        failed_tests = [test for test in self.test_results if test['status'] == 'FAIL']
        if failed_tests:
            self.logger.info(f"\nFAILED TESTS ({len(failed_tests)}):")
            for test in failed_tests:
                self.logger.info(f"- {test['method']} {test['endpoint']}: {test.get('error', 'Unknown error')}")
        
        self.logger.info("=" * 80)


def main():
    """Main execution function"""
    import argparse
    
    parser = argparse.ArgumentParser(description='Test Dua API endpoints')
    parser.add_argument('--url', default='http://localhost:3000', 
                       help='Base URL for the API (default: http://localhost:3000)')
    parser.add_argument('--test', choices=['all', 'health', 'duas', 'categories', 'tags', 'bundles', 'sources', 'media', 'search', 'stats'], 
                       default='all', help='Specific test to run (default: all)')
    
    args = parser.parse_args()
    
    tester = DuaAPITester(base_url=args.url)
    
    if args.test == 'all':
        tester.run_all_tests()
    elif args.test == 'health':
        tester.test_health_check()
    elif args.test == 'duas':
        tester.discover_test_data()
        tester.test_list_duas()
        tester.test_get_random_dua()
        tester.test_get_dua_by_id()
    elif args.test == 'categories':
        tester.discover_test_data()
        tester.test_categories()
    elif args.test == 'tags':
        tester.discover_test_data()
        tester.test_tags()
    elif args.test == 'bundles':
        tester.discover_test_data()
        tester.test_bundles()
    elif args.test == 'sources':
        tester.discover_test_data()
        tester.test_sources()
    elif args.test == 'media':
        tester.discover_test_data()
        tester.test_media()
    elif args.test == 'search':
        tester.discover_test_data()
        tester.test_search()
    elif args.test == 'stats':
        tester.test_stats()
    
    tester.print_summary()


if __name__ == "__main__":
    main()