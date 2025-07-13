<template>
  <div class="pricing-calculator">
    <div class="calculator-header">
      <h2>
        <el-icon><Calculator /></el-icon>
        Smart Pricing Calculator
      </h2>
      <p class="calculator-description">
        Find the perfect tier for your needs with our intelligent pricing engine
      </p>
    </div>

    <div class="calculator-content">
      <div class="calculator-inputs">
        <div class="input-section">
          <h3>Your Usage Requirements</h3>
          
          <div class="input-group">
            <label for="storage-slider">
              <el-icon><Folder /></el-icon>
              Storage Needed
            </label>
            <div class="slider-container">
              <el-slider
                id="storage-slider"
                v-model="storageNeeded"
                :min="1"
                :max="10000"
                :step="100"
                :format-tooltip="formatStorageTooltip"
                @change="recalculatePricing"
              />
              <div class="slider-value">
                {{ formatBytes(storageNeeded * 1024 * 1024 * 1024) }}
              </div>
            </div>
          </div>

          <div class="input-group">
            <label for="uploads-slider">
              <el-icon><Upload /></el-icon>
              Monthly Uploads
            </label>
            <div class="slider-container">
              <el-slider
                id="uploads-slider"
                v-model="monthlyUploads"
                :min="1"
                :max="1000"
                :step="10"
                :format-tooltip="formatUploadTooltip"
                @change="recalculatePricing"
              />
              <div class="slider-value">
                {{ formatBytes(monthlyUploads * 1024 * 1024 * 1024) }}
              </div>
            </div>
          </div>

          <div class="input-group">
            <label for="contribution-slider">
              <el-icon><Share /></el-icon>
              Contribution Capacity
            </label>
            <div class="slider-container">
              <el-slider
                id="contribution-slider"
                v-model="contributionCapacity"
                :min="0"
                :max="50000"
                :step="1000"
                :format-tooltip="formatContributionTooltip"
                @change="recalculatePricing"
              />
              <div class="slider-value">
                {{ formatBytes(contributionCapacity * 1024 * 1024 * 1024) }}
              </div>
            </div>
          </div>

          <div class="input-group">
            <label for="priority-select">
              <el-icon><Timer /></el-icon>
              Priority Level
            </label>
            <el-select
              id="priority-select"
              v-model="priorityLevel"
              placeholder="Select priority"
              @change="recalculatePricing"
            >
              <el-option
                v-for="priority in priorityOptions"
                :key="priority.value"
                :label="priority.label"
                :value="priority.value"
              >
                <span>{{ priority.label }}</span>
                <span class="priority-description">{{ priority.description }}</span>
              </el-option>
            </el-select>
          </div>

          <div class="usage-patterns">
            <h4>Usage Patterns</h4>
            <div class="pattern-toggles">
              <el-checkbox
                v-model="usagePatterns.bursty"
                @change="recalculatePricing"
              >
                Bursty Usage
                <el-tooltip content="Occasional high usage periods">
                  <el-icon><QuestionFilled /></el-icon>
                </el-tooltip>
              </el-checkbox>
              
              <el-checkbox
                v-model="usagePatterns.steady"
                @change="recalculatePricing"
              >
                Steady Growth
                <el-tooltip content="Predictable growth over time">
                  <el-icon><QuestionFilled /></el-icon>
                </el-tooltip>
              </el-checkbox>
              
              <el-checkbox
                v-model="usagePatterns.backup"
                @change="recalculatePricing"
              >
                Backup Focused
                <el-tooltip content="Primarily for backup storage">
                  <el-icon><QuestionFilled /></el-icon>
                </el-tooltip>
              </el-checkbox>
            </div>
          </div>
        </div>

        <div class="regional-settings">
          <h4>Regional Settings</h4>
          <el-select
            v-model="selectedRegion"
            placeholder="Select region"
            @change="recalculatePricing"
          >
            <el-option
              v-for="region in regions"
              :key="region.code"
              :label="region.name"
              :value="region.code"
            >
              <span>{{ region.name }}</span>
              <span class="region-multiplier">{{ region.multiplier }}x</span>
            </el-option>
          </el-select>
        </div>
      </div>

      <div class="calculator-results">
        <div class="results-header">
          <h3>Recommended Tiers</h3>
          <div class="savings-indicator" v-if="bestSavings > 0">
            <el-icon><Money /></el-icon>
            Save up to ${{ bestSavings.toFixed(2) }}/month
          </div>
        </div>

        <div class="tier-comparison">
          <div 
            v-for="tier in calculatedTiers"
            :key="tier.name"
            class="tier-option"
            :class="{ 
              'recommended': tier.recommended,
              'insufficient': tier.insufficient,
              'overpriced': tier.overpriced
            }"
          >
            <div class="tier-header">
              <div class="tier-name">
                <h4>{{ tier.name }}</h4>
                <el-tag
                  v-if="tier.recommended"
                  type="success"
                  size="small"
                >
                  Recommended
                </el-tag>
                <el-tag
                  v-if="tier.insufficient"
                  type="danger"
                  size="small"
                >
                  Insufficient
                </el-tag>
                <el-tag
                  v-if="tier.overpriced"
                  type="warning"
                  size="small"
                >
                  Overpriced
                </el-tag>
              </div>
              
              <div class="tier-pricing">
                <div class="price-display">
                  <span class="base-price" v-if="tier.basePrice !== tier.actualPrice">
                    ${{ tier.basePrice.toFixed(2) }}
                  </span>
                  <span class="actual-price">
                    ${{ tier.actualPrice.toFixed(2) }}
                  </span>
                  <span class="price-period">/month</span>
                </div>
                
                <div class="price-breakdown" v-if="tier.discount > 0">
                  <span class="discount-amount">
                    -${{ tier.discount.toFixed(2) }} discount
                  </span>
                </div>
              </div>
            </div>

            <div class="tier-specs">
              <div class="spec-item">
                <el-icon><Folder /></el-icon>
                <span>{{ formatBytes(tier.storage) }} Storage</span>
              </div>
              <div class="spec-item">
                <el-icon><Connection /></el-icon>
                <span>{{ formatBytes(tier.bandwidth) }} Bandwidth</span>
              </div>
              <div class="spec-item">
                <el-icon><Timer /></el-icon>
                <span>{{ tier.priority }} Priority</span>
              </div>
              <div class="spec-item" v-if="tier.burstStorage > 0">
                <el-icon><TrendCharts /></el-icon>
                <span>{{ formatBytes(tier.burstStorage) }} Burst</span>
              </div>
            </div>

            <div class="tier-benefits">
              <div class="benefits-title">Included Features</div>
              <ul class="benefits-list">
                <li v-for="benefit in tier.benefits" :key="benefit">
                  <el-icon><Check /></el-icon>
                  {{ benefit }}
                </li>
              </ul>
            </div>

            <div class="tier-analysis">
              <div class="efficiency-score">
                <div class="score-label">Efficiency Score</div>
                <div class="score-bar">
                  <div 
                    class="score-fill"
                    :style="{ 
                      width: `${tier.efficiency}%`,
                      backgroundColor: getEfficiencyColor(tier.efficiency)
                    }"
                  ></div>
                  <span class="score-value">{{ tier.efficiency }}%</span>
                </div>
              </div>

              <div class="cost-analysis">
                <div class="cost-per-gb">
                  ${{ (tier.actualPrice / (tier.storage / (1024**3))).toFixed(4) }}/GB
                </div>
                <div class="yearly-cost">
                  ${{ (tier.actualPrice * 12).toFixed(2) }}/year
                </div>
              </div>
            </div>

            <div class="tier-actions">
              <el-button
                v-if="tier.recommended"
                type="primary"
                size="large"
                @click="selectTier(tier)"
              >
                Choose This Tier
              </el-button>
              <el-button
                v-else
                type="default"
                size="large"
                @click="selectTier(tier)"
              >
                Select
              </el-button>
            </div>
          </div>
        </div>

        <div class="prediction-insights">
          <h4>Smart Predictions</h4>
          <div class="insights-grid">
            <div class="insight-card" v-for="insight in pricingInsights" :key="insight.id">
              <div class="insight-icon">{{ insight.icon }}</div>
              <div class="insight-content">
                <h5>{{ insight.title }}</h5>
                <p>{{ insight.description }}</p>
                <div class="insight-confidence">
                  Confidence: {{ insight.confidence }}%
                </div>
              </div>
            </div>
          </div>
        </div>

        <div class="dynamic-pricing-info">
          <h4>
            <el-icon><TrendCharts /></el-icon>
            Dynamic Pricing Factors
          </h4>
          <div class="pricing-factors">
            <div 
              v-for="factor in pricingFactors"
              :key="factor.name"
              class="factor-item"
            >
              <div class="factor-name">{{ factor.name }}</div>
              <div class="factor-impact" :class="factor.impact > 0 ? 'positive' : 'negative'">
                {{ factor.impact > 0 ? '+' : '' }}{{ (factor.impact * 100).toFixed(1) }}%
              </div>
              <div class="factor-description">{{ factor.description }}</div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div class="calculator-footer">
      <div class="disclaimer">
        <el-icon><InfoFilled /></el-icon>
        Prices are estimates based on current market conditions and may vary.
        Dynamic pricing adjusts based on network supply and demand.
      </div>
      
      <div class="actions">
        <el-button @click="resetCalculator">Reset</el-button>
        <el-button type="primary" @click="saveConfiguration">
          Save Configuration
        </el-button>
      </div>
    </div>
  </div>
</template>

<script>
import { ref, computed, onMounted, watch } from 'vue'
import { useEconomyStore } from '@/stores/economy'
import { useDynamicPricing } from '@/composables/useDynamicPricing'
import { formatBytes, formatCurrency } from '@/utils/formatters'
import { 
  Calculator, 
  Folder, 
  Upload, 
  Share, 
  Timer, 
  Money, 
  Connection, 
  TrendCharts, 
  Check, 
  QuestionFilled, 
  InfoFilled 
} from '@element-plus/icons-vue'

export default {
  name: 'InteractivePricingCalculator',
  components: {
    Calculator,
    Folder,
    Upload,
    Share,
    Timer,
    Money,
    Connection,
    TrendCharts,
    Check,
    QuestionFilled,
    InfoFilled
  },
  setup() {
    const economyStore = useEconomyStore()
    const { getDynamicPricing, getPricingFactors, getPredictiveInsights } = useDynamicPricing()

    // Input state
    const storageNeeded = ref(100) // GB
    const monthlyUploads = ref(50) // GB
    const contributionCapacity = ref(200) // GB
    const priorityLevel = ref('standard')
    const selectedRegion = ref('us-west')
    const usagePatterns = ref({
      bursty: false,
      steady: true,
      backup: false
    })

    // Computed pricing
    const calculatedTiers = ref([])
    const pricingFactors = ref([])
    const pricingInsights = ref([])
    const bestSavings = ref(0)

    // Configuration
    const priorityOptions = [
      { 
        value: 'low', 
        label: 'Low Priority', 
        description: 'Best effort processing',
        multiplier: 0.8
      },
      { 
        value: 'standard', 
        label: 'Standard Priority', 
        description: 'Normal processing speed',
        multiplier: 1.0
      },
      { 
        value: 'high', 
        label: 'High Priority', 
        description: 'Faster processing',
        multiplier: 1.3
      },
      { 
        value: 'premium', 
        label: 'Premium Priority', 
        description: 'Highest priority processing',
        multiplier: 1.8
      }
    ]

    const regions = [
      { code: 'us-west', name: 'US West', multiplier: 1.0 },
      { code: 'us-east', name: 'US East', multiplier: 1.0 },
      { code: 'eu-west', name: 'EU West', multiplier: 1.1 },
      { code: 'ap-southeast', name: 'Asia Pacific', multiplier: 1.2 },
      { code: 'global', name: 'Global', multiplier: 1.3 }
    ]

    const baseTiers = [
      {
        id: 'free',
        name: 'Free',
        basePrice: 0,
        storage: 5 * 1024**3, // 5GB
        bandwidth: 10 * 1024**3, // 10GB
        priority: 'Low',
        burstStorage: 1 * 1024**3, // 1GB
        benefits: ['Basic encryption', 'Community support', 'Standard reliability']
      },
      {
        id: 'basic',
        name: 'Basic',
        basePrice: 9.99,
        storage: 100 * 1024**3, // 100GB
        bandwidth: 200 * 1024**3, // 200GB
        priority: 'Standard',
        burstStorage: 20 * 1024**3, // 20GB
        benefits: ['Enhanced encryption', 'Priority support', 'Advanced reliability', 'Basic analytics']
      },
      {
        id: 'pro',
        name: 'Pro',
        basePrice: 29.99,
        storage: 1000 * 1024**3, // 1TB
        bandwidth: 2000 * 1024**3, // 2TB
        priority: 'High',
        burstStorage: 200 * 1024**3, // 200GB
        benefits: ['Advanced encryption', 'Premium support', 'High reliability', 'Advanced analytics', 'API access']
      },
      {
        id: 'enterprise',
        name: 'Enterprise',
        basePrice: 99.99,
        storage: 10000 * 1024**3, // 10TB
        bandwidth: 20000 * 1024**3, // 20TB
        priority: 'Premium',
        burstStorage: 2000 * 1024**3, // 2TB
        benefits: ['Enterprise encryption', '24/7 support', 'Maximum reliability', 'Custom analytics', 'Full API access', 'Custom integrations']
      }
    ]

    // Methods
    const formatStorageTooltip = (value) => `${formatBytes(value * 1024**3)}`
    const formatUploadTooltip = (value) => `${formatBytes(value * 1024**3)}`
    const formatContributionTooltip = (value) => `${formatBytes(value * 1024**3)}`

    const getEfficiencyColor = (efficiency) => {
      if (efficiency >= 80) return '#67C23A'
      if (efficiency >= 60) return '#E6A23C'
      return '#F56C6C'
    }

    const calculateTierEfficiency = (tier, requirements) => {
      const storageEfficiency = Math.min(100, (requirements.storage / tier.storage) * 100)
      const bandwidthEfficiency = Math.min(100, (requirements.bandwidth / tier.bandwidth) * 100)
      const priceEfficiency = Math.max(0, 100 - ((tier.actualPrice - tier.basePrice) / tier.basePrice * 100))
      
      return Math.round((storageEfficiency + bandwidthEfficiency + priceEfficiency) / 3)
    }

    const calculateDynamicPrice = async (tier, requirements) => {
      try {
        const dynamicPricing = await getDynamicPricing(tier.id, selectedRegion.value)
        
        let price = tier.basePrice
        let discount = 0

        // Apply dynamic pricing multiplier
        price *= dynamicPricing.multiplier

        // Apply regional adjustment
        const region = regions.find(r => r.code === selectedRegion.value)
        if (region) {
          price *= region.multiplier
        }

        // Apply priority multiplier
        const priority = priorityOptions.find(p => p.value === priorityLevel.value)
        if (priority) {
          price *= priority.multiplier
        }

        // Apply contribution discount
        if (contributionCapacity.value > 0) {
          const contributionRatio = contributionCapacity.value / (storageNeeded.value || 1)
          if (contributionRatio >= 2) {
            discount = price * 0.1 // 10% discount for 2:1 ratio
          }
          if (contributionRatio >= 4) {
            discount = price * 0.2 // 20% discount for 4:1 ratio
          }
        }

        // Apply usage pattern adjustments
        if (usagePatterns.value.steady) {
          discount += price * 0.05 // 5% discount for steady usage
        }
        if (usagePatterns.value.backup) {
          discount += price * 0.1 // 10% discount for backup usage
        }
        if (usagePatterns.value.bursty) {
          price *= 1.1 // 10% increase for bursty usage
        }

        return {
          actualPrice: Math.max(0, price - discount),
          discount,
          factors: dynamicPricing.factors
        }
      } catch (error) {
        console.error('Failed to calculate dynamic price:', error)
        return {
          actualPrice: tier.basePrice,
          discount: 0,
          factors: []
        }
      }
    }

    const recalculatePricing = async () => {
      const requirements = {
        storage: storageNeeded.value * 1024**3,
        bandwidth: monthlyUploads.value * 1024**3,
        priority: priorityLevel.value,
        region: selectedRegion.value,
        patterns: usagePatterns.value
      }

      const updatedTiers = []
      let maxSavings = 0

      for (const tier of baseTiers) {
        const pricing = await calculateDynamicPrice(tier, requirements)
        
        const updatedTier = {
          ...tier,
          actualPrice: pricing.actualPrice,
          discount: pricing.discount,
          efficiency: calculateTierEfficiency(tier, requirements),
          insufficient: tier.storage < requirements.storage,
          overpriced: pricing.actualPrice > tier.basePrice * 1.5,
          recommended: false
        }

        // Calculate savings compared to base price
        const savings = tier.basePrice - pricing.actualPrice
        if (savings > maxSavings) {
          maxSavings = savings
        }

        updatedTiers.push(updatedTier)
      }

      // Find the recommended tier
      const suitableTiers = updatedTiers.filter(tier => !tier.insufficient && !tier.overpriced)
      if (suitableTiers.length > 0) {
        // Recommend the tier with highest efficiency among suitable tiers
        const recommended = suitableTiers.reduce((best, current) => 
          current.efficiency > best.efficiency ? current : best
        )
        recommended.recommended = true
      }

      calculatedTiers.value = updatedTiers
      bestSavings.value = maxSavings

      // Update pricing factors
      const allFactors = updatedTiers.flatMap(tier => pricing.factors || [])
      pricingFactors.value = await getPricingFactors(requirements)

      // Update insights
      pricingInsights.value = await getPredictiveInsights(requirements, updatedTiers)
    }

    const selectTier = (tier) => {
      economyStore.selectTier(tier.id)
      ElMessage.success(`Selected ${tier.name} tier`)
    }

    const resetCalculator = () => {
      storageNeeded.value = 100
      monthlyUploads.value = 50
      contributionCapacity.value = 200
      priorityLevel.value = 'standard'
      selectedRegion.value = 'us-west'
      usagePatterns.value = {
        bursty: false,
        steady: true,
        backup: false
      }
      recalculatePricing()
    }

    const saveConfiguration = () => {
      const config = {
        storage: storageNeeded.value,
        uploads: monthlyUploads.value,
        contribution: contributionCapacity.value,
        priority: priorityLevel.value,
        region: selectedRegion.value,
        patterns: usagePatterns.value
      }
      
      localStorage.setItem('datamesh-pricing-config', JSON.stringify(config))
      ElMessage.success('Configuration saved')
    }

    const loadConfiguration = () => {
      const saved = localStorage.getItem('datamesh-pricing-config')
      if (saved) {
        const config = JSON.parse(saved)
        storageNeeded.value = config.storage || 100
        monthlyUploads.value = config.uploads || 50
        contributionCapacity.value = config.contribution || 200
        priorityLevel.value = config.priority || 'standard'
        selectedRegion.value = config.region || 'us-west'
        usagePatterns.value = config.patterns || { bursty: false, steady: true, backup: false }
      }
    }

    // Lifecycle
    onMounted(async () => {
      loadConfiguration()
      await recalculatePricing()
    })

    // Watch for changes
    watch([storageNeeded, monthlyUploads, contributionCapacity, priorityLevel, selectedRegion], 
      recalculatePricing, { deep: true })

    return {
      // Input state
      storageNeeded,
      monthlyUploads,
      contributionCapacity,
      priorityLevel,
      selectedRegion,
      usagePatterns,
      
      // Computed results
      calculatedTiers,
      pricingFactors,
      pricingInsights,
      bestSavings,
      
      // Configuration
      priorityOptions,
      regions,
      
      // Methods
      formatStorageTooltip,
      formatUploadTooltip,
      formatContributionTooltip,
      getEfficiencyColor,
      recalculatePricing,
      selectTier,
      resetCalculator,
      saveConfiguration,
      
      // Utilities
      formatBytes,
      formatCurrency
    }
  }
}
</script>

<style scoped>
.pricing-calculator {
  max-width: 1200px;
  margin: 0 auto;
  padding: 20px;
}

.calculator-header {
  text-align: center;
  margin-bottom: 30px;
}

.calculator-header h2 {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
  font-size: 2rem;
  color: #303133;
  margin-bottom: 10px;
}

.calculator-description {
  color: #606266;
  font-size: 1.1rem;
}

.calculator-content {
  display: grid;
  grid-template-columns: 1fr 2fr;
  gap: 30px;
  margin-bottom: 30px;
}

.calculator-inputs {
  background: #f8f9fa;
  padding: 25px;
  border-radius: 12px;
  height: fit-content;
}

.input-section h3 {
  margin-bottom: 20px;
  color: #303133;
}

.input-group {
  margin-bottom: 25px;
}

.input-group label {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 10px;
  font-weight: 500;
  color: #303133;
}

.slider-container {
  display: flex;
  align-items: center;
  gap: 15px;
}

.slider-container .el-slider {
  flex: 1;
}

.slider-value {
  min-width: 80px;
  text-align: right;
  font-weight: 600;
  color: #409EFF;
}

.priority-description {
  font-size: 0.8rem;
  color: #909399;
  margin-left: 10px;
}

.usage-patterns {
  margin-top: 25px;
}

.usage-patterns h4 {
  margin-bottom: 15px;
  color: #303133;
}

.pattern-toggles {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.pattern-toggles .el-checkbox {
  display: flex;
  align-items: center;
  gap: 5px;
}

.regional-settings {
  margin-top: 25px;
}

.regional-settings h4 {
  margin-bottom: 15px;
  color: #303133;
}

.region-multiplier {
  float: right;
  color: #909399;
  font-size: 0.9rem;
}

.calculator-results {
  background: white;
  border-radius: 12px;
  border: 1px solid #e4e7ed;
  overflow: hidden;
}

.results-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px 25px;
  background: #f8f9fa;
  border-bottom: 1px solid #e4e7ed;
}

.results-header h3 {
  margin: 0;
  color: #303133;
}

.savings-indicator {
  display: flex;
  align-items: center;
  gap: 8px;
  color: #67C23A;
  font-weight: 600;
}

.tier-comparison {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
  gap: 20px;
  padding: 25px;
}

.tier-option {
  border: 2px solid #e4e7ed;
  border-radius: 12px;
  padding: 20px;
  transition: all 0.3s ease;
  background: white;
}

.tier-option:hover {
  border-color: #409EFF;
  transform: translateY(-2px);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.1);
}

.tier-option.recommended {
  border-color: #67C23A;
  background: #f0f9ff;
}

.tier-option.insufficient {
  border-color: #F56C6C;
  background: #fef0f0;
}

.tier-option.overpriced {
  border-color: #E6A23C;
  background: #fdf6ec;
}

.tier-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 15px;
}

.tier-name h4 {
  margin: 0 0 5px 0;
  color: #303133;
}

.tier-pricing {
  text-align: right;
}

.price-display {
  display: flex;
  align-items: baseline;
  gap: 8px;
  justify-content: flex-end;
}

.base-price {
  text-decoration: line-through;
  color: #909399;
  font-size: 0.9rem;
}

.actual-price {
  font-size: 1.5rem;
  font-weight: 700;
  color: #303133;
}

.price-period {
  color: #606266;
  font-size: 0.9rem;
}

.discount-amount {
  color: #67C23A;
  font-size: 0.8rem;
  font-weight: 600;
}

.tier-specs {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 10px;
  margin-bottom: 15px;
}

.spec-item {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 0.9rem;
  color: #606266;
}

.tier-benefits {
  margin-bottom: 15px;
}

.benefits-title {
  font-weight: 600;
  color: #303133;
  margin-bottom: 10px;
}

.benefits-list {
  list-style: none;
  padding: 0;
  margin: 0;
}

.benefits-list li {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 5px;
  font-size: 0.85rem;
  color: #606266;
}

.benefits-list li .el-icon {
  color: #67C23A;
}

.tier-analysis {
  margin-bottom: 20px;
}

.efficiency-score {
  margin-bottom: 10px;
}

.score-label {
  font-size: 0.85rem;
  color: #606266;
  margin-bottom: 5px;
}

.score-bar {
  position: relative;
  height: 8px;
  background: #e4e7ed;
  border-radius: 4px;
  overflow: hidden;
}

.score-fill {
  height: 100%;
  transition: width 0.3s ease;
}

.score-value {
  position: absolute;
  right: 0;
  top: -18px;
  font-size: 0.8rem;
  color: #303133;
  font-weight: 600;
}

.cost-analysis {
  display: flex;
  justify-content: space-between;
  font-size: 0.8rem;
  color: #909399;
}

.tier-actions {
  text-align: center;
}

.tier-actions .el-button {
  width: 100%;
}

.prediction-insights {
  padding: 25px;
  background: #f8f9fa;
  border-top: 1px solid #e4e7ed;
}

.prediction-insights h4 {
  margin-bottom: 15px;
  color: #303133;
}

.insights-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
  gap: 15px;
}

.insight-card {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 15px;
  background: white;
  border-radius: 8px;
  border: 1px solid #e4e7ed;
}

.insight-icon {
  font-size: 1.5rem;
  flex-shrink: 0;
}

.insight-content h5 {
  margin: 0 0 5px 0;
  color: #303133;
}

.insight-content p {
  margin: 0 0 8px 0;
  font-size: 0.9rem;
  color: #606266;
}

.insight-confidence {
  font-size: 0.8rem;
  color: #909399;
}

.dynamic-pricing-info {
  padding: 25px;
  border-top: 1px solid #e4e7ed;
}

.dynamic-pricing-info h4 {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 15px;
  color: #303133;
}

.pricing-factors {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 15px;
}

.factor-item {
  display: flex;
  flex-direction: column;
  padding: 12px;
  background: #f8f9fa;
  border-radius: 8px;
  border-left: 4px solid #409EFF;
}

.factor-name {
  font-weight: 600;
  color: #303133;
  margin-bottom: 5px;
}

.factor-impact {
  font-size: 1.1rem;
  font-weight: 700;
  margin-bottom: 5px;
}

.factor-impact.positive {
  color: #67C23A;
}

.factor-impact.negative {
  color: #F56C6C;
}

.factor-description {
  font-size: 0.85rem;
  color: #606266;
}

.calculator-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px;
  background: #f8f9fa;
  border-radius: 8px;
  margin-top: 20px;
}

.disclaimer {
  display: flex;
  align-items: center;
  gap: 8px;
  color: #909399;
  font-size: 0.9rem;
}

.actions {
  display: flex;
  gap: 10px;
}

/* Responsive Design */
@media (max-width: 768px) {
  .calculator-content {
    grid-template-columns: 1fr;
  }
  
  .tier-comparison {
    grid-template-columns: 1fr;
  }
  
  .tier-specs {
    grid-template-columns: 1fr;
  }
  
  .insights-grid {
    grid-template-columns: 1fr;
  }
  
  .pricing-factors {
    grid-template-columns: 1fr;
  }
  
  .calculator-footer {
    flex-direction: column;
    gap: 15px;
    text-align: center;
  }
}
</style>
