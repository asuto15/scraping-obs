export interface Env {
	GITHUB_TOKEN: string;
}

const OWNER = 'asuto15';
const REPO = 'scraping-obs';
const WORKFLOW_FILE = 'scrape.yml';
const REF = 'main';

export default {
	async scheduled(event: ScheduledEvent, env: Env, ctx: ExecutionContext): Promise<void> {
		const url = `https://api.github.com/repos/${OWNER}/${REPO}/actions/workflows/${WORKFLOW_FILE}/dispatches`;

		const res = await fetch(url, {
			method: 'POST',
			headers: {
				Authorization: `Bearer ${env.GITHUB_TOKEN}`,
				Accept: 'application/vnd.github+json',
				'Content-Type': 'application/json',
				'User-Agent': 'github-dispatch-worker',
			},
			body: JSON.stringify({
				ref: REF,
			}),
		});

		if (!res.ok) {
			console.error('dispatch failed:', await res.text());
			throw new Error(`GitHub dispatch failed: ${res.status}`);
		} else {
			console.log('GitHub workflow_dispatch sent successfully');
		}
	},
};
